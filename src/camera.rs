use crate::capture::{add_input, finish_writer, host_time, new_writer, video_settings};
use anyhow::{Context as _, Result, anyhow, bail};
use block2::RcBlock;
use dispatch2::DispatchQueue;
use objc2::rc::Retained;
use objc2::runtime::{Bool, NSObject, ProtocolObject};
use objc2::{AnyThread, ClassType, DefinedClass, define_class, msg_send};
use objc2_av_foundation::{
    AVAssetWriter, AVAssetWriterInput, AVAssetWriterInputPixelBufferAdaptor, AVAuthorizationStatus, AVCaptureConnection,
    AVCaptureDevice, AVCaptureDeviceInput, AVCaptureOutput, AVCaptureSession, AVCaptureSessionPreset1280x720,
    AVCaptureAudioDataOutput, AVCaptureAudioDataOutputSampleBufferDelegate, AVCaptureVideoDataOutput, AVCaptureVideoDataOutputSampleBufferDelegate, AVMediaTypeAudio, AVMediaTypeVideo, AVVideoCodecTypeH264,
};
use objc2_core_media::CMSampleBuffer;
use objc2_core_video::{CVPixelBufferGetHeight, CVPixelBufferGetWidth};
use objc2_foundation::{NSObjectProtocol, NSString};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Track {
    writer: Retained<AVAssetWriter>,
    input: Retained<AVAssetWriterInput>,
    adaptor: Retained<AVAssetWriterInputPixelBufferAdaptor>,
    audio: Option<Retained<AVAssetWriterInput>>,
}

/// The writer is created on the first frame, once the camera's frame size is known.
struct CamWriter {
    path: PathBuf,
    track: Option<Track>,
    start: Option<f64>,
    last: f64,
    failed: Option<String>,
    mic: bool,
    dimensions: (u32, u32),
    finished: bool,
}

unsafe impl Send for CamWriter {}

impl CamWriter {
    fn append_audio(&mut self, sample: &CMSampleBuffer) {
        let Some(start) = self.start else { return };
        if self.finished || self.failed.is_some() || unsafe { sample.presentation_time_stamp().seconds() } < start { return; }
        let Some(input) = self.track.as_ref().and_then(|t| t.audio.as_ref()) else { return };
        unsafe {
            if input.isReadyForMoreMediaData() && !input.appendSampleBuffer(sample) {
                self.failed = Some("microphone sample rejected".into());
            }
        }
    }

    fn append(&mut self, sample: &CMSampleBuffer) {
        if self.finished || self.failed.is_some() {
            return;
        }
        let Some(buffer) = (unsafe { sample.image_buffer() }) else { return };
        let host = unsafe { sample.presentation_time_stamp().seconds() };
        if self.track.is_none() {
            let (w, h) = (CVPixelBufferGetWidth(&buffer), CVPixelBufferGetHeight(&buffer));
            let made = new_writer(&self.path).and_then(|writer| {
                let settings = video_settings(w, h, unsafe { AVVideoCodecTypeH264.unwrap() }, 8_000_000, 30, false);
                let input = add_input(&writer, unsafe { AVMediaTypeVideo.unwrap() }, &settings)?;
                let audio = self.mic.then(|| add_input(&writer, unsafe { AVMediaTypeAudio.unwrap() }, &crate::audio::aac_settings(1, 96_000))).transpose()?;
                let adaptor = unsafe {
                    AVAssetWriterInputPixelBufferAdaptor::assetWriterInputPixelBufferAdaptorWithAssetWriterInput_sourcePixelBufferAttributes(&input, None)
                };
                unsafe {
                    if !writer.startWriting() {
                        bail!("camera writer did not start");
                    }
                    writer.startSessionAtSourceTime(host_time(host));
                }
                Ok(Track { writer, input, adaptor, audio })
            });
            match made {
                Ok(track) => {
                    self.track = Some(track);
                    self.start = Some(host);
                    self.dimensions = (w as u32, h as u32);
                }
                Err(e) => {
                    self.failed = Some(format!("{e:#}"));
                    return;
                }
            }
        }
        let track = self.track.as_ref().unwrap();
        if host <= self.last {
            return;
        }
        unsafe {
            if track.input.isReadyForMoreMediaData() && !track.adaptor.appendPixelBuffer_withPresentationTime(&buffer, host_time(host)) {
                self.failed = Some("camera frame rejected".into());
                return;
            }
        }
        self.last = host;
    }
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = Arc<Mutex<CamWriter>>]
    struct CameraSink;

    unsafe impl NSObjectProtocol for CameraSink {}

    unsafe impl AVCaptureAudioDataOutputSampleBufferDelegate for CameraSink {
        #[unsafe(method(captureOutput:didOutputSampleBuffer:fromConnection:))]
        fn did_audio(&self, output: &AVCaptureOutput, sample: &CMSampleBuffer, _connection: &AVCaptureConnection) {
            if output.isKindOfClass(AVCaptureAudioDataOutput::class()) {
                self.ivars().lock().unwrap().append_audio(sample);
            } else {
                self.ivars().lock().unwrap().append(sample);
            }
        }
    }

    // The audio and video delegate protocols share the same Objective-C selector.
    unsafe impl AVCaptureVideoDataOutputSampleBufferDelegate for CameraSink {}
);

impl CameraSink {
    fn new(writer: Arc<Mutex<CamWriter>>) -> Retained<Self> {
        let this = Self::alloc().set_ivars(writer);
        unsafe { msg_send![super(this), init] }
    }
}

/// Asks for camera or microphone access up front, so the prompt never lands mid-recording.
pub fn ensure_access(mic: bool) -> Result<()> {
    let (media, name) = unsafe {
        if mic { (AVMediaTypeAudio.unwrap(), "Microphone") } else { (AVMediaTypeVideo.unwrap(), "Camera") }
    };
    match unsafe { AVCaptureDevice::authorizationStatusForMediaType(media) } {
        AVAuthorizationStatus::Authorized => Ok(()),
        AVAuthorizationStatus::NotDetermined => {
            let (tx, rx) = mpsc::channel();
            let handler = RcBlock::new(move |granted: Bool| {
                let _ = tx.send(granted.as_bool());
            });
            unsafe { AVCaptureDevice::requestAccessForMediaType_completionHandler(media, &handler) };
            match rx.recv_timeout(Duration::from_secs(120)) {
                Ok(true) => Ok(()),
                Ok(false) => bail!("{name} access was declined. Enable Recorder in System Settings → Privacy & Security → {name}, or turn off {name} in Inputs to record without it."),
                Err(_) => bail!("Still waiting for {name} access. Respond to the macOS permission dialog, then try recording again."),
            }
        }
        AVAuthorizationStatus::Restricted => bail!("{name} access is restricted by this Mac's settings. Turn off {name} in Inputs to record without it."),
        _ => bail!("{name} access is off. Enable Recorder in System Settings → Privacy & Security → {name}, or turn off {name} in Inputs to record without it."),
    }
}

pub struct CameraHandle {
    session: Retained<AVCaptureSession>,
    writer: Arc<Mutex<CamWriter>>,
    _sink: Retained<CameraSink>,
    _output: Retained<AVCaptureVideoDataOutput>,
    _audio: Option<Retained<AVCaptureAudioDataOutput>>,
}

unsafe impl Send for CameraHandle {}

impl CameraHandle {
    pub fn start(path: &Path, device: Option<&str>) -> Result<Self> {
        Self::start_event(path, device, false, None)
    }

    /// Starts selected devices and waits for the first video frame.
    pub fn start_event(path: &Path, device: Option<&str>, mic: bool, microphone: Option<&str>) -> Result<Self> {
        ensure_access(false)?;
        if mic { ensure_access(true)?; }
        let writer = Arc::new(Mutex::new(CamWriter { path: path.to_path_buf(), track: None, start: None, last: f64::MIN, failed: None, mic, dimensions: (0, 0), finished: false }));
        let sink = CameraSink::new(writer.clone());
        unsafe {
            let device = resolve_device(device, false)?;
            let input = AVCaptureDeviceInput::deviceInputWithDevice_error(&device)
                .map_err(|e| anyhow!("open camera: {}", e.localizedDescription()))?;
            let session = AVCaptureSession::new();
            session.beginConfiguration();
            if session.canSetSessionPreset(AVCaptureSessionPreset1280x720) {
                session.setSessionPreset(AVCaptureSessionPreset1280x720);
            }
            if !session.canAddInput(&input) {
                bail!("cannot use the camera");
            }
            session.addInput(&input);
            let output = AVCaptureVideoDataOutput::new();
            output.setAlwaysDiscardsLateVideoFrames(true);
            let queue = DispatchQueue::new("recorder.camera", None);
            output.setSampleBufferDelegate_queue(Some(ProtocolObject::from_ref(&*sink)), Some(&queue));
            if !session.canAddOutput(&output) {
                bail!("cannot read camera frames");
            }
            session.addOutput(&output);
            let audio = if mic {
                let device = resolve_device(microphone, true)?;
                let input = AVCaptureDeviceInput::deviceInputWithDevice_error(&device)
                    .map_err(|e| anyhow!("open microphone: {}", e.localizedDescription()))?;
                if !session.canAddInput(&input) { bail!("cannot use the microphone"); }
                session.addInput(&input);
                let output = AVCaptureAudioDataOutput::new();
                output.setSampleBufferDelegate_queue(Some(ProtocolObject::from_ref(&*sink)), Some(&queue));
                if !session.canAddOutput(&output) { bail!("cannot read microphone audio"); }
                session.addOutput(&output);
                Some(output)
            } else { None };
            session.commitConfiguration();
            session.startRunning();

            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                {
                    let w = writer.lock().unwrap();
                    if let Some(e) = w.failed.clone() {
                        drop(w);
                        session.stopRunning();
                        bail!("camera: {e}");
                    }
                    if w.start.is_some() {
                        break;
                    }
                }
                if Instant::now() > deadline {
                    session.stopRunning();
                    bail!("the camera sent no frames");
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(Self { session, writer, _sink: sink, _output: output, _audio: audio })
        }
    }

    /// Report encoder errors or an unplugged camera while recording.
    pub fn error(&self) -> Option<String> {
        let w = self.writer.lock().unwrap();
        w.failed.clone().or_else(|| (w.start.is_some() && crate::capture::host_now() - w.last > 5.0).then(|| "The camera stopped sending frames. Check its connection.".into()))
    }

    /// Stops capture, finalizes its media, and returns timing and dimensions.
    pub fn finish(self) -> Result<CameraRecording> {
        self.writer.lock().unwrap().finished = true;
        unsafe { self.session.stopRunning() };
        let mut w = self.writer.lock().unwrap();
        let start = w.start.context("the camera sent no frames")?;
        if let Some(e) = w.failed.take() {
            bail!("camera: {e}");
        }
        let track = w.track.take().context("camera writer missing")?;
        unsafe {
            track.input.markAsFinished();
            if let Some(audio) = &track.audio { audio.markAsFinished(); }
            track.writer.endSessionAtSourceTime(host_time(w.last));
        }
        finish_writer(&track.writer)?;
        Ok(CameraRecording { start, duration: w.last - start, width: w.dimensions.0, height: w.dimensions.1 })
    }
}

impl Drop for CameraHandle {
    fn drop(&mut self) {
        self.writer.lock().unwrap().finished = true;
        unsafe { self.session.stopRunning(); }
    }
}

pub struct CameraRecording {
    pub start: f64,
    pub duration: f64,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
}

pub fn list_devices(mic: bool) -> Vec<DeviceInfo> {
    unsafe {
        let media = if mic { AVMediaTypeAudio.unwrap() } else { AVMediaTypeVideo.unwrap() };
        #[allow(deprecated)]
        AVCaptureDevice::devicesWithMediaType(media).iter()
            .map(|d| DeviceInfo { id: d.uniqueID().to_string(), name: d.localizedName().to_string() }).collect()
    }
}

pub(crate) fn resolve_device(id: Option<&str>, mic: bool) -> Result<Retained<AVCaptureDevice>> {
    unsafe {
        let media = if mic { AVMediaTypeAudio.unwrap() } else { AVMediaTypeVideo.unwrap() };
        let device = match id {
            Some(id) => AVCaptureDevice::deviceWithUniqueID(&NSString::from_str(id)),
            None => AVCaptureDevice::defaultDeviceWithMediaType(media),
        }.context(if mic { "Selected microphone is unavailable. Refresh inputs and choose a connected microphone." } else { "Selected camera is unavailable. Refresh inputs and choose a connected webcam." })?;
        if !device.hasMediaType(media) { bail!("selected device does not support the requested input"); }
        Ok(device)
    }
}
