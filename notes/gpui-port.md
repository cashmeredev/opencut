# GPUI desktop port — architecture & plan

Captured 2026-07-28. Owner decisions this session:

- **Full 1:1 port** of the editor to GPUI. Linux-first; macOS/Windows not targeted for now, BSD parked (GPUI support unproven).
- **In-process ffmpeg** via `ffmpeg-next` for decode AND encode. No subprocess.
- **Licensing is not a constraint** — x264 (GPL) is the H.264 encoder; quality over AppImage-ability. The documented OpenH264 dead end (ignores bitrate, undershoots ~10x) is bypassed entirely.
- Web version stays and keeps working; the browser editor remains a feature.

## Session log

### Session 1 (2026-07-28) — foundation COMPLETE

`cargo test --workspace` green, ~300 tests, 0 failures. Checkpoints:
`e837e750` (web session work), `f5a82152` (port foundation), `32438ce2`
(domain crates).

Done and verified:

| Crate | Tests | Contents |
|---|---|---|
| `scene` | 4 | full model, serde round-trip vs web SerializedProject, factory (v32 defaults) |
| `transfer` | 7 | `.ocp` zip container, cross-compat with web fflate BOTH directions (fixture in tests/fixtures) |
| `storage` | 7 | filesystem ProjectStore, media store, `.ocp` import/export, id-collision suffix |
| `media` | 10 | ffmpeg-next 8 decode: frames (seek+forward cache), audio ranges, thumbnails, waveform |
| `playback` | 10 | wall-clock anchored, frame-rounded clock, volume/mute |
| `timeline` | 45 | placement, resize/ripple, split, snap, retime, audio separation |
| `animation` | 73 | bezier solver, sampling, keyframe ops, split/clamp/clone, param defs |
| `commands` | 9 | Command trait, Batch, history, 19 concrete commands, freeze-frame 1:1 |
| `audio` | 22 | audible collection, range mixing, mastering limiter, rms buckets |
| `text` | 19 | cosmic-text layout/measure/raster, backgrounds, decorations |
| `graphics` | 52 | FULL CSS gradient parser, tiny-skia raster, 4 shape defs |
| `effect-defs` | 16 | blur def, registry, params→passes |
| `mask-defs` | 19 | all 9 mask renderers via tiny-skia |
| `renderer` | 6 | node model + scene builder (resolve/frame-descriptor NOT yet) |

Environment: flake devshell has rust + GPUI native deps + ffmpeg 8.1.2 dev +
bindgen args. GPUI builds; headless wgpu compositing + texture readback
proven (compositor/tests/offscreen_render.rs). gpu crate wasm32 compat
re-verified after readback addition. Desktop app: GPUI projects screen
(list/create/delete/open via ProjectStore) builds; editor screen is a stub.

Run anything: `nix develop --command cargo test -p <crate>` (cargo is NOT
on bare PATH; `nix develop` required, also provides ffmpeg/bindgen env).

### Known gaps / next session starts here

1. **renderer resolve + frame-descriptor** — the integration hub. Port
   `apps/web/src/services/renderer/{resolve.ts, compositor/frame-descriptor.ts}`:
   node tree → resolve at time t (uses animation sampling, media frames via
   `media` crate, text measure via `text`, graphic params via `graphics`,
   mask artifacts via `mask-defs`) → `compositor::FrameDescriptor` + texture
   uploads. All leaf deps now exist. START HERE.
2. **GPUI preview element** — paint compositor output (readback → gpui::Image)
   in the editor screen; wire playback clock → resolve → render loop.
3. **Audio note**: volume params are DECIBELS (gain=10^(db/20), clamp
   [-60,20], missing = unity) — audio crate encodes this. maintain_pitch=
   true returns typed Error::MaintainPitchUnsupported (no soundtouch v1).
4. **Text mask unification**: mask-defs has its own cosmic-text engine in
   text_mask.rs; swap to sibling `text` crate's TextEngine if desired.
5. **cosmic-text gotcha**: Attrs::letter_spacing is EM units; web px value
   must be divided by scaled font size (already applied in both crates).
6. **encode crate** — x264 export (compositor frames → ffmpeg encode →
   mux). ffmpeg-next is proven; nothing written yet.
7. **Not ported by design**: sounds/stickers/effects PANELS (flagged off in
   web; data still renders), storage migrations (desktop starts at v32),
   clipboard commands (browser APIs), keyframe/effect/mask command classes
   (framework proven; port when editor state needs them), curve-bridge.ts
   (canvas UI), soundtouch pitch preservation, conic gradients (not in web
   parser either).
8. **Web-only side effects intentionally dropped**: EditorCore selection
   snapshots in history, ripple reactors, insert-element canvas/fps
   auto-adjust, hidden-element audio exclusion follows contract not web
   (web keeps hidden video audio in export — revisit at export time).
9. Agents do NOT use git; Main commits at green checkpoints. Boundary rule:
   agents never write outside their own crate dir.

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
