# MP3 Lab ⚡

Open-source audio clip editor with a retro media-player aesthetic for Windows, Linux, and Android.

**Project status: source integration / builds pending.** No executable downloads are available yet; check Releases only after CI validation.

## Features

- Select and preview audio clips
- Fade in / fade out
- Optional loudness normalization and manual gain
- Peak limiting
- Export 256 kbps MP3

## Implementation

- Windows and Linux: Rust desktop application (egui/eframe + FFmpeg tools).
- Android: native Android application using an FFmpeg audio encoding dependency.

The prepared source package includes a cross-platform GitHub Actions workflow; source integration and compilation remain to be completed. Third-party FFmpeg licensing must be reviewed before binary distribution.

## License

MIT for original MP3 Lab source code; third-party components retain their own licenses.
