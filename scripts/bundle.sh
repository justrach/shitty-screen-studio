#!/usr/bin/env bash
# LaunchServices owns the signed bundle so macOS permissions belong to Recorder.
set -euo pipefail
cd "$(dirname "$0")/.."
NAME="Recorder"
FLAVOR="${RECORDER_BUILD_FLAVOR:-dev}"
case "$FLAVOR" in
  dev) BUNDLE_ID="recorder.codegraff.app.dev" ;;
  release) BUNDLE_ID="recorder.codegraff.app" ;;
  *) echo "RECORDER_BUILD_FLAVOR must be dev or release" >&2; exit 2 ;;
esac
DEST="${DEST:-$HOME/Applications}"
APP="$DEST/$NAME.app"
VERSION="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
IDENTITY="${RECORDER_DEV_CODESIGN_IDENTITY:-${HARNESS_DEV_CODESIGN_IDENTITY:-${SIGN_IDENTITY:-}}}"
if [[ -z "$IDENTITY" ]]; then
  IDENTITY="$(security find-identity -v -p codesigning 2>/dev/null | sed -n 's/.*"\(Apple Development:[^"]*\)".*/\1/p' | head -1)"
fi
cargo build --release
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/recorder "$APP/Contents/MacOS/recorder"
GRAFF_SRC="${GRAFF_BINARY:-$(command -v graff || true)}"
if [[ -n "$GRAFF_SRC" ]]; then
  mkdir -p "$APP/Contents/Resources/bin"
  install -m 755 "$GRAFF_SRC" "$APP/Contents/Resources/bin/graff"
  cp vendor/codegraff/LICENSE "$APP/Contents/Resources/Codegraff-LICENSE"
  codesign --force --sign "${IDENTITY:--}" "$APP/Contents/Resources/bin/graff"
fi
ICON_ROOT="$(mktemp -d)"
trap 'rm -rf "$ICON_ROOT"' EXIT
ICONSET="$ICON_ROOT/AppIcon.iconset"
mkdir -p "$ICONSET"
swift scripts/app-icon.swift 1024 "$ICON_ROOT/master.png"
for size in 16 32 128 256 512; do
  sips -z "$size" "$size" "$ICON_ROOT/master.png" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
  retina=$((size * 2))
  sips -z "$retina" "$retina" "$ICON_ROOT/master.png" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>$NAME</string>
  <key>CFBundleDisplayName</key><string>$NAME</string>
  <key>CFBundleIdentifier</key><string>$BUNDLE_ID</string>
  <key>CFBundleExecutable</key><string>recorder</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSScreenCaptureUsageDescription</key><string>Record your selected display, window, or area and save it in your recording library.</string>
  <key>NSMicrophoneUsageDescription</key><string>Add your voice when Microphone is enabled. Turn it off to record without microphone audio.</string>
  <key>NSCameraUsageDescription</key><string>Add a camera overlay when Camera is enabled. Turn it off to record without a camera overlay.</string>
  <key>NSHumanReadableCopyright</key><string>Based on Shitty Screen Studio by Li Yang (@hewliyang). Harness UI by Standard Harness, Rach Pradhan and Yu Xi Lim.</string>
</dict></plist>
PLIST
codesign --force --sign "${IDENTITY:--}" --identifier "$BUNDLE_ID" "$APP"
codesign --verify --strict "$APP"
if [[ -z "$IDENTITY" ]]; then
  echo "warning: no Apple Development identity found. Set up your certificate as in Harness; rebuilds may reset recording permission." >&2
else
  echo "signed Recorder with $IDENTITY ($BUNDLE_ID)" >&2
fi
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$APP"
echo "$APP"
