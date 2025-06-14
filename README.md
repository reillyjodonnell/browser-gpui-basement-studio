## Browser gpui basement studio

Implemented a gpui POC from Jose's tweet https://x.com/ragojose/status/1928512106380263723

<img width="1512" alt="Screenshot 2025-06-12 at 8 49 52 AM" src="https://github.com/user-attachments/assets/2e703b36-ab89-4100-8292-c637d9ee91dc" />

## Instructions

Mac OS:
`cargo cef-build-debug`
`./target/debug/browser.app/Contents/MacOS/browser`

## Deps

rust >= 1.87.0
xcode

### architecture

```
GPUI (Rust UI Framework)
↓ (FFI)
Custom Rust/C++ Bridge Layer
↓
Chromium Content API (C++)
```
