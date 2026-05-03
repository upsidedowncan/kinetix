# Video Import / Preview Notes

## What was failing
- Imported videos had valid duration metadata from Rust, but preview playback showed the crossed-out play icon.
- The preview component was repeatedly calling `video.load()` inside a reactive block:
  - This caused frequent source reloads and unstable playback behavior.

## Relevant Tauri docs (v2)
- `convertFileSrc()` should be used for local file paths loaded by WebView media elements.
- Asset protocol must be enabled in `tauri.conf.json`:
  - `app.security.assetProtocol.enable = true`
  - `app.security.assetProtocol.scope = ["**"]` (or a tighter scope in production)
- Ref: https://v2.tauri.app/reference/javascript/api/namespacecore/ (`convertFileSrc` section)

## Implemented fix
1. Use Rust-backed proxy generation via FFmpeg for all imported videos.
2. The proxy uses H.264 Baseline profile and AAC audio, which is guaranteed to be compatible with most WebViews (including WebKitGTK on Linux).
3. Proxy files are stored in the app's cache directory under a `proxies` folder.
4. The UI uses the proxy URL for preview and timeline playback.
5. The original file path is preserved in the project state for final high-quality export.
6. Keyed video element (`{#key videoSrc}`) with nested `<source>` ensures source changes remount cleanly.
7. Removed reactive `video.load()` loop from `VideoPreview.svelte`.

## Added UX improvements
- Import loading state (`Importing...`) and disabled import buttons during proxy generation.
- Inline preview playback error panel when embedded decoder fails.
- Current filename shown in preview footer.
- Automatic proxy generation on import from both Sidebar and Preview.

## Requirements
- `ffmpeg` must be installed on the system path for proxy generation to work.

## Follow-ups
- If playback still fails on Linux for specific codecs, this is usually a system WebView codec support issue.
- Consider optional transcode/proxy generation (FFmpeg) for guaranteed preview compatibility.
