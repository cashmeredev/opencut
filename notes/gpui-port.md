# GPUI desktop port — architecture & plan

Captured 2026-07-28. Owner decisions this session:

- **Full 1:1 port** of the editor to GPUI. Linux-first; macOS/Windows not targeted for now, BSD parked (GPUI support unproven).
- **In-process ffmpeg** via `ffmpeg-next` for decode AND encode. No subprocess.
- **Licensing is not a constraint** — x264 (GPL) is the H.264 encoder; quality over AppImage-ability. The documented OpenH264 dead end (ignores bitrate, undershoots ~10x) is bypassed entirely.
- Web version stays and keeps working; the browser editor remains a feature.

## Why the port

Browser inconsistency (WebGPU quirks, encoder behavior varying by browser)
keeps producing walls. A native app gives one controlled runtime: wgpu +
ffmpeg, one decoder, one encoder, one compositor.

## What "backend" means here

There is no server worth porting: the Next.js backend is an auth scaffold
(no flows implemented), a health route, and a sounds proxy (sounds are
feature-flagged off). The real "backend" is the offline project store
(IndexedDB) — on desktop it becomes a plain filesystem layout per project.
So: no Rust server, no API. Everything lives in the app process.

## Crate map

Existing, reused natively as-is (already target-agnostic):

- `rust/crates/time` — media time, frame rate, timecode
- `rust/crates/compositor` + `effects` + `masks` + `gpu` — wgpu compositor,
  WGSL effect pipeline, SDF mask feathering. The web drives this through
  wasm; desktop links it directly.

New crates:

- `scene` — project/scene graph model, serde, `.ocp` project.json compat
  (round-trip proven against the web's SerializedProject shape).
- `timeline` — tracks, elements, placement, snapping, split/trim/resize,
  ripple, grouping. Pure logic, no UI.
- `animation` — keyframes, bezier interpolation, channel sampling.
- `commands` — undo/redo command stack (ports `apps/web/src/commands`).
- `media` — ffmpeg-next demux/decode, frame pipeline to GPU textures,
  thumbnails, waveform extraction.
- `audio` — decode, mixing, rate stretching, mastering.
- `encode` — export pipeline: compositor frames → x264/VP9, muxing.
- `transfer` — `.ocp` zip container build/parse (cross-compat with the web's
  fflate archives proven in both directions).
- `storage` — project dirs on disk, per-project media store, autosave,
  `.ocp` import/export (replaces IndexedDB; stays offline-first).
- `text` — text layout/measure/raster via cosmic-text (ports
  `apps/web/src/text`).
- `graphics` — gradient parser + shape raster via tiny-skia (ports
  `apps/web/src/graphics` + `gradients`).
- `effect-defs` — effect definitions registry: params → WGSL pass
  descriptors (ports `apps/web/src/effects/definitions`).
- `mask-defs` — mask shape renderers via tiny-skia (ports
  `apps/web/src/masks/builtin` + `freeform`).
- `renderer` — scene → node tree → resolve at time t → compositor
  FrameDescriptor (ports `apps/web/src/services/renderer`). Integration hub;
  starts once the leaf crates land.
- `playback` — clock, frame scheduling, audio sync.

`apps/desktop` (GPUI) holds UI only; all logic lives in crates so the web
version can adopt them later via wasm where it makes sense.

## Compositor → GPUI bridge

GPUI does not expose its internal GPU device. v1 approach: the compositor
owns its own wgpu device, renders the frame to an offscreen texture,
reads back, and paints it as a `gpui::Image` in a custom element. ~8 MB
per 1080p frame — fine for preview; revisit if profiling says otherwise.

## Milestones (each runnable)

1. **Environment** — nix devshell with rust + GPUI native deps + ffmpeg dev.
2. **Domain model** — `scene` crate: full project/scene/element model
   ported from `apps/web/src/timeline/types.ts` + scene managers, `.ocp`
   round-trip compat proven against web-generated archives.
3. **Timeline core** — `timeline`, `animation`, `commands` crates with
   ported unit tests (web test suite is the behavior spec).
4. **Media** — `media`/`audio` crates: decode → texture, waveform,
   thumbnails.
5. **Compositor bridge** — scene resolve → wgpu frame → GPUI preview.
6. **GPUI UI** — projects screen, editor layout, timeline UI (drag, trim,
   split, snap, ripple), preview overlays/selection, all panels.
7. **Export** — `encode` crate + dialog, x264 quality ladder.
8. **Features** — versions, voiceover (cpal), clipboard, keybindings,
   `.ocp` import/export.
9. **Packaging** — .desktop entry, binary distribution. AppImage optional.

## Port rules

- Web behavior is the spec. Where the web has unit tests, port the test
  with the code.
- Flagged-off web features (sounds, stickers, effects domains) are NOT
  ported — they are slated for deletion upstream of us.
- `.ocp` archives must round-trip between web and desktop.
- No parallel divergence in shared crates: `compositor`/`time` changes must
  keep compiling for `wasm32`.
