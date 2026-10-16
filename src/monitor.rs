//! Disposable AVFoundation preflight capture. Never creates a writer or saves media.
use crate::camera::{ensure_access, resolve_device};
use crate::player::Surface;
use anyhow::{Result, anyhow, bail};
use dispatch2::{DispatchQueue, DispatchRetained};
use objc2::rc::Retained;
use objc2::runtime::{NSObject, ProtocolObject};
use objc2::{AnyThread, ClassType, DefinedClass, define_class, msg_send};
use objc2_av_foundation::{
    AVCaptureAudioDataOutput, AVCaptureAudioDataOutputSampleBufferDelegate, AVCaptureConnection,
    AVCaptureDeviceInput, AVCaptureOutput, AVCaptureSession, AVCaptureSessionPreset1280x720,
    AVCaptureVideoDataOutput, AVCaptureVideoDataOutputSampleBufferDelegate, AVMediaTypeVideo,
};
use objc2_core_media::CMSampleBuffer;
use objc2_core_video::{CVPixelBufferGetHeight, CVPixelBufferGetWidth};
use objc2_foundation::{NSNumber, NSObjectProtocol, NSString};
use std::sync::{Arc, Mutex, mpsc};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Config {
    pub camera: bool,
    pub mic: bool,
    pub camera_device: Option<String>,
    pub mic_device: Option<String>,
    pub mirror: bool,
}

pub struct Snapshot {
    pub frame: Option<Surface>,
    pub dimensions: (usize, usize),
    pub average: f32,
    pub peak: f32,
    pub max_peak: f32,
    pub error: Option<String>,
    pub running: bool,
    started: Instant,
    video_at: Option<Instant>,
    audio_at: Option<Instant>,
    clipped_at: Option<Instant>,
    frames: u64,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            frame: None, dimensions: (0, 0), average: -60., peak: -60., max_peak: -60.,
            error: None, running: false, started: Instant::now(), video_at: None,
            audio_at: None, clipped_at: None, frames: 0,
        }
    }
}

impl Snapshot {
    pub fn video_live(&self) -> bool {
        self.running && self.video_at.is_some_and(|t| t.elapsed() < Duration::from_secs(2))
    }

    pub fn audio_live(&self) -> bool {
        self.running && self.audio_at.is_some_and(|t| t.elapsed() < Duration::from_secs(1))
    }

    pub fn clipping(&self) -> bool {
        self.audio_live() && self.clipped_at.is_some_and(|t| t.elapsed() < Duration::from_secs(2))
    }

    pub fn fps(&self) -> f64 {
        self.frames as f64 / self.started.elapsed().as_secs_f64().max(0.001)
    }

    pub fn waiting_too_long(&self, camera: bool, mic: bool) -> bool {
        self.running && self.started.elapsed() > Duration::from_secs(5)
            && ((camera && !self.video_live()) || (mic && !self.audio_live()))
    }
}

pub fn meter_fraction(db: f32) -> f32 {
    if db.is_finite() { ((db + 60.) / 60.).clamp(0., 1.) } else { 0. }
}

fn level(db: f32) -> f32 {
    if db.is_finite() { db.clamp(-60., 0.) } else { -60. }
}

type Shared = Arc<Mutex<Snapshot>>;

// One selector implements both delegate protocols, just like the recording sink.
define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = Shared]
    struct MonitorSink;

    unsafe impl NSObjectProtocol for MonitorSink {}

    unsafe impl AVCaptureAudioDataOutputSampleBufferDelegate for MonitorSink {
        #[unsafe(method(captureOutput:didOutputSampleBuffer:fromConnection:))]
        fn sample(&self, output: &AVCaptureOutput, sample: &CMSampleBuffer, connection: &AVCaptureConnection) {
            let now = Instant::now();
            if output.isKindOfClass(AVCaptureAudioDataOutput::class()) {
                // AVFoundation reports calibrated dBFS per channel; show the loudest channel.
                let channels = unsafe { connection.audioChannels() };
                let mut average = -60.0_f32;
                let mut peak = -60.0_f32;
                for channel in channels.iter() {
                    average = average.max(level(unsafe { channel.averagePowerLevel() }));
                    peak = peak.max(level(unsafe { channel.peakHoldLevel() }));
                }
                if channels.is_empty() { return; }
                let mut s = self.ivars().lock().unwrap();
                s.average = average;
                s.peak = peak;
                s.max_peak = s.max_peak.max(peak);
                s.audio_at = Some(now);
                if peak >= -1. { s.clipped_at = Some(now); }
            } else if let Some(buffer) = unsafe { sample.image_buffer() } {
                let mut s = self.ivars().lock().unwrap();
                s.dimensions = (CVPixelBufferGetWidth(&buffer), CVPixelBufferGetHeight(&buffer));
                s.frame = Some(Surface(buffer));
                s.video_at = Some(now);
                s.frames += 1;
            }
        }
    }

    unsafe impl AVCaptureVideoDataOutputSampleBufferDelegate for MonitorSink {}
);

impl MonitorSink {
    fn new(shared: Shared) -> Retained<Self> {
        let this = Self::alloc().set_ivars(shared);
        unsafe { msg_send![super(this), init] }
    }
}

struct Session {
    session: Retained<AVCaptureSession>,
    _sink: Retained<MonitorSink>,
    _video: Option<Retained<AVCaptureVideoDataOutput>>,
    _audio: Option<Retained<AVCaptureAudioDataOutput>>,
    queue: DispatchRetained<DispatchQueue>,
}

impl Session {
    // All session configuration, start, and stop stay on the same dedicated worker.
    fn start(config: &Config, shared: Shared) -> Result<Self> {
        unsafe {
            let session = AVCaptureSession::new();
            let sink = MonitorSink::new(shared.clone());
            let queue = DispatchQueue::new("recorder.preflight.samples", None);
            session.beginConfiguration();
            if config.camera && session.canSetSessionPreset(AVCaptureSessionPreset1280x720) {
                session.setSessionPreset(AVCaptureSessionPreset1280x720);
            }
            for (enabled, mic, id) in [(config.camera, false, config.camera_device.as_deref()), (config.mic, true, config.mic_device.as_deref())] {
                if !enabled { continue; }
                let device = resolve_device(id, mic)?;
                let input = AVCaptureDeviceInput::deviceInputWithDevice_error(&device)
                    .map_err(|e| anyhow!("Open input: {}", e.localizedDescription()))?;
                if !session.canAddInput(&input) { bail!("macOS could not add the selected input. It may be in use by another app."); }
                session.addInput(&input);
            }
            let video = if config.camera {
                let output = AVCaptureVideoDataOutput::new();
                output.setAlwaysDiscardsLateVideoFrames(true);
                output.setSampleBufferDelegate_queue(Some(ProtocolObject::from_ref(&*sink)), Some(&queue));
                if !session.canAddOutput(&output) { bail!("macOS could not create the webcam preview."); }
                session.addOutput(&output);
                // GPUI's Metal surface renderer requires full-range NV12 (420f),
                // not video-range NV12 (420v), and asserts on other formats.
                let nv12 = u32::from_be_bytes(*b"420f");
                if !output.availableVideoCVPixelFormatTypes().iter().any(|n| n.unsignedIntValue() == nv12) {
                    bail!("This camera does not provide full-range NV12 for native preview. Choose another webcam.");
                }
                let key = NSString::from_str("PixelFormatType");
                let value = NSNumber::new_u32(nv12);
                output.setVideoSettings(Some(&crate::capture::dict(&[(&key, value.as_ref())])));
                let connection = output.connectionWithMediaType(AVMediaTypeVideo.unwrap())
                    .ok_or_else(|| anyhow!("macOS could not connect the webcam preview."))?;
                if connection.isVideoMirroringSupported() {
                    connection.setAutomaticallyAdjustsVideoMirroring(false);
                    connection.setVideoMirrored(config.mirror);
                } else if config.mirror {
                    bail!("This webcam cannot mirror its preview. Turn off Mirror preview and retry the input check.");
                }
                Some(output)
            } else { None };
            let audio = if config.mic {
                let output = AVCaptureAudioDataOutput::new();
                output.setSampleBufferDelegate_queue(Some(ProtocolObject::from_ref(&*sink)), Some(&queue));
                if !session.canAddOutput(&output) { bail!("macOS could not create the microphone meter."); }
                session.addOutput(&output);
                Some(output)
            } else { None };
            session.commitConfiguration();
            // Construct the guard first so failures also stop the capture session.
            let result = Self { session, _sink: sink, _video: video, _audio: audio, queue };
            result.session.startRunning();
            if !result.session.isRunning() { bail!("macOS could not start preview. Check that the selected devices are connected and available."); }
            let mut s = shared.lock().unwrap();
            s.running = true;
            s.started = Instant::now();
            Ok(result)
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe { self.session.stopRunning(); }
        if let Some(video) = &self._video {
            unsafe { video.setSampleBufferDelegate_queue(None, None); }
        }
        if let Some(audio) = &self._audio {
            unsafe { audio.setSampleBufferDelegate_queue(None, None); }
        }
        // Stop acknowledgements must follow every in-flight sample callback.
        // This owner thread is distinct from the serial sample queue.
        self.queue.exec_sync(|| {});
    }
}

struct Command {
    config: Option<Config>,
    shared: Shared,
    stopped: Option<mpsc::Sender<()>>,
    generation: u64,
}

/// A single serial worker prevents overlapping sessions during rapid input changes.
/// A stop acknowledgement lets recording wait until preflight releases the hardware.
pub struct Monitor {
    tx: Option<mpsc::Sender<Command>>,
    pub shared: Shared,
    generation: Arc<AtomicU64>,
}

impl Monitor {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Command>();
        let generation = Arc::new(AtomicU64::new(0));
        let current = generation.clone();
        std::thread::Builder::new().name("preflight".into()).spawn(move || {
            let mut session = None;
            while let Ok(command) = rx.recv() {
                drop(session.take());
                if let Some(stopped) = command.stopped { let _ = stopped.send(()); }
                if command.generation != current.load(Ordering::SeqCst) { continue; }
                let Some(config) = command.config else { continue };
                let result = (|| {
                    if config.camera { ensure_access(false)?; }
                    if command.generation != current.load(Ordering::SeqCst) { return Ok(None); }
                    if config.mic { ensure_access(true)?; }
                    // A permission dialog can outlive a Stop or a device change.
                    if command.generation != current.load(Ordering::SeqCst) { return Ok(None); }
                    Session::start(&config, command.shared.clone()).map(Some)
                })();
                match result {
                    Ok(active) => session = active,
                    Err(e) => command.shared.lock().unwrap().error = Some(format!("{e:#}")),
                }
            }
        }).expect("spawn preflight worker");
        Self { tx: Some(tx), shared: Arc::new(Mutex::new(Snapshot::default())), generation }
    }

    pub fn configure(&mut self, config: Config) {
        self.send(Some(config), None);
    }

    pub fn stop(&mut self) -> mpsc::Receiver<()> {
        let (tx, rx) = mpsc::channel();
        self.send(None, Some(tx));
        rx
    }

    fn send(&mut self, config: Option<Config>, stopped: Option<mpsc::Sender<()>>) {
        // Each generation gets its own state: a late callback cannot refill a stopped preview.
        self.shared = Arc::new(Mutex::new(Snapshot::default()));
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let _ = self.tx.as_ref().unwrap().send(Command { config, shared: self.shared.clone(), stopped, generation });
    }

    pub fn reset_peak(&self) {
        let mut s = self.shared.lock().unwrap();
        s.max_peak = -60.;
        s.clipped_at = None;
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.tx.take(); // Disconnecting stops the session on its owner thread.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meter_uses_a_clamped_dbfs_scale() {
        assert_eq!(meter_fraction(-60.), 0.);
        assert_eq!(meter_fraction(-30.), 0.5);
        assert_eq!(meter_fraction(0.), 1.);
        assert_eq!(meter_fraction(12.), 1.);
        assert_eq!(meter_fraction(f32::NAN), 0.);
        assert_eq!(level(f32::NEG_INFINITY), -60.);
    }

    #[test]
    fn stale_inputs_are_not_reported_as_live_or_clipping() {
        let mut s = Snapshot::default();
        s.running = true;
        s.started = Instant::now() - Duration::from_secs(10);
        s.video_at = Some(Instant::now() - Duration::from_secs(6));
        s.audio_at = s.video_at;
        s.clipped_at = Some(Instant::now());
        assert!(!s.video_live());
        assert!(!s.audio_live());
        assert!(!s.clipping());
        assert!(s.waiting_too_long(true, true));
    }

    #[test]
    fn idle_worker_acknowledges_hardware_release_without_permissions() {
        let mut monitor = Monitor::new();
        monitor.stop().recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(!monitor.shared.lock().unwrap().running);
    }
}
