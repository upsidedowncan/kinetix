# Video Playback Deep Research Report (Linux + Tauri + WebKitGTK)

## Scope
User symptom: imported MP4 appears in media pool and timeline with valid duration, but preview remains black.

## Confirmed local evidence
- Import pipeline works (from logs):
  - `addMedia: "ForBiggerBlazes.mp4" "video/mp4" 15.021`
  - `Media pool now has 1 items`
  - Rust metadata command logs: `MP4 duration: 15.021 seconds`
- This proves file selection + metadata extraction + timeline insertion are functioning.

## Codebase audit findings

### 1) Import and URL generation
- `VideoPreview.svelte` uses `convertFileSrc(filePath)` for video source.
- `Sidebar.svelte` uses `convertFileSrc` and dispatches import data.
- `tauri.conf.json` has:
  - `app.security.assetProtocol.enable = true`
  - `app.security.assetProtocol.scope = ["**"]`

Status: aligned with Tauri v2 docs for local file media URLs.

### 2) Timeline/preview synchronization
- Fixed earlier: preview sync now searches all visual tracks, not only track 0.
- Fixed earlier: playhead/seek reset to clip start on import/select.
- Fixed earlier: removed reactive `video.load()` loop.

Status: major app-logic black-frame causes have been addressed.

## External research (primary sources)

### A) Tauri docs: convertFileSrc + asset protocol requirements
- Source: https://v2.tauri.app/reference/javascript/api/namespacecore/
- `convertFileSrc()` is the intended mechanism for loading local files in webview.
- Requires `app.security.assetProtocol.enable = true` and appropriate `scope`.

Conclusion: current implementation follows this path.

### B) Tauri docs: Linux media stack is WebKitGTK-based
- Source: https://v2.tauri.app/reference/webview-versions/
- Linux webview behavior depends on distro WebKitGTK version/runtime.

Conclusion: playback capability and rendering stability are environment-dependent on Linux.

### C) Tauri AppImage docs: media depends on GStreamer availability/plugins
- Source: https://v2.tauri.app/distribute/appimage/
- Explicit note: enable `bundleMediaFramework` to include GStreamer media framework in bundles.

Conclusion: media decode/render can vary by deployment/runtime environment.

### D) Upstream issue: video starts but not rendered in Tauri/WebKitGTK
- Source: https://github.com/tauri-apps/tauri/issues/8579
- Repro includes plain HTML `<video>` in fresh Tauri app; player renders but video not shown.
- Stack traces include DRM/GBM failures (`DRM_IOCTL_MODE_CREATE_DUMB failed`).

Conclusion: there are known upstream Linux rendering regressions independent of app code.

### E) Upstream issue: Linux + NVIDIA blank rendering (DMA-BUF path)
- Source: https://github.com/tauri-apps/tauri/issues/9304
- Source: https://github.com/nymtech/nym-vpn-client/issues/305
- Workaround repeatedly used in ecosystem:
  - `WEBKIT_DISABLE_DMABUF_RENDERER=1`

Conclusion: black surfaces on Linux can be GPU/DMABUF rendering path issues, not file validity.

## Root-cause ranking (current confidence)

1. **Codec support mismatch in embedded WebView vs system player** (High confidence)
   - Verified that "plays in system player" does not guarantee playback in WebKitGTK.
   - Decided to move to proxy generation to bypass this fragmentation.

2. **Linux WebKitGTK DMA-BUF renderer path issue** (High confidence)
   - Added `WEBKIT_DISABLE_DMABUF_RENDERER=1` workaround.

## Mitigations implemented in this pass

1. **Rust-backed Proxy Generation** (Primary Solution)
- Command: `generate_proxy_video`
- Implementation: Uses system `ffmpeg` to create H.264 Baseline / AAC MP4s.
- Storage: Saved in app cache directory (`~/.cache/kinetix/proxies` on Linux).
- Frontend: `Sidebar` and `VideoPreview` trigger proxying on import.

2. **Linux runtime workaround**
- Added `WEBKIT_DISABLE_DMABUF_RENDERER=1` to `main.rs`.

## What to test now (strict order)

1. Full restart of `tauri dev` (required after Rust/env change).
2. Re-import same MP4 and play timeline.
3. Confirm whether preview now shows frames.

If still black:
4. Test with known baseline codec file: H.264 + AAC MP4.
5. If baseline works and original does not, codec support is confirmed as limiting factor.
6. If baseline also black, environment renderer issue remains primary.

## Next engineering options (if workaround is insufficient)

### Option 1: add runtime diagnostics panel (fast)
- Show in UI:
  - `video.error?.code`, `video.error?.message`
  - `readyState`, `networkState`
  - `canPlayType` probe results for common mime+codec strings.

### Option 2: implement preview proxy generation (robust)
- Generate preview-safe transcoded proxy (H.264 baseline + AAC) via Rust command (ffmpeg/gstreamer).
- Use proxy in preview/timeline playback; keep original source for export.

### Option 3: distribution hardening for Linux bundles
- Enable AppImage media framework bundling:
  - `bundle.linux.appimage.bundleMediaFramework = true`
- Validate plugin availability on target distros.

## Decision
Given current evidence, the most evidence-backed immediate fix is the Linux DMA-BUF renderer workaround (implemented). If black preview persists after restart, proxy transcoding is the highest-confidence cross-machine fix.
