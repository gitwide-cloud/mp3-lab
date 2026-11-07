# ⚡ MP3 Lab

![MP3 Lab lightning logo](assets/mp3-lab-logo.svg)

Free, open-source clip editor with a retro Winamp-inspired interface. MP3 Lab has a Rust desktop edition for Windows and Linux and a native Android edition.

## Download builds

**[GitHub Releases](https://github.com/gitwide-cloud/mp3-lab/releases)**

Every push to `main` starts GitHub Actions jobs for Windows, Linux and Android. The `v0.1.0-preview` release is created **only if all three builds succeed**; if there is no release, check [Actions](https://github.com/gitwide-cloud/mp3-lab/actions) for failures. Releases are preview builds, not verified production software.

Expected files:
- `mp3-lab-windows.zip`: Windows desktop executable
- `mp3-lab-linux.zip`: Linux desktop executable
- `mp3-lab.apk`: Android ARM64 debug-signed installer (Android 8+)

## Features

- Set clip start and end points
- Preview original or processed audio
- Fade in and fade out
- Normalize loudness to -16 LUFS
- Adjust gain -24 dB to +12 dB
- Apply a peak limiter
- Export MP3 at constant 256 kbps

## Build requirements

### Windows / Linux
```sh
cd desktop
cargo build --release
```

Install **FFmpeg** and ensure `ffmpeg`, `ffprobe` and `ffplay` are available on PATH. Windows users can put these executables next to `mp3-lab.exe`. Linux users may install FFmpeg via their package manager.

### Android
Use JDK 17, Android SDK 35, Gradle 8.9 and run:

```sh
cd android
gradle :app:assembleDebug
```

The app uses `dev.ffmpegkit-maintained:ffmpeg-kit-audio:8.1.7` to encode audio locally. APK support is ARM64 only.

## Development status and limitations

- CI builds may require fixes; successful builds do not prove playback, file handling or export correctness on every device.
- Desktop has a clip selection timeline rather than the waveform shown in the early concept mockup.
- Android is written in Java; the desktop apps are written in Rust.
- The Android app has native start/end sliders, not the illustrated desktop waveform.
- Linux desktop depends on system libraries and FFmpeg; binaries are not AppImages.
- This project does not bundle FFmpeg desktop executables.

## Licensing

Original MP3 Lab code is MIT licensed. FFmpegKit and FFmpeg components have separate LGPL/GPL terms depending on how they are distributed and configured. Review third-party license obligations before redistributing builds. Not affiliated with Winamp.

## Contributing

Issues and pull requests welcome. Functional testing and accessibility improvements are especially valuable.
