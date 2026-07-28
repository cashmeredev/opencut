# Roadmap & session notes

Working notes for the fork's development direction. Captured 2026-07-26,
updated 2026-07-28 (GPUI desktop port greenlit — see notes/gpui-port.md).

## Done (2026-07-28 session)

- Freeze-duration prompt removed; freeze frame uses a fixed 3s (store, dialog, and test deleted).
- Ripple-extend: dragging the right edge of a static element (image/freeze still/text, i.e. no `sourceDuration`) past its neighbor pushes the whole right-hand chain on that track. Same undo step as the resize; live during drag. Video/audio keep clamping. Core: `timeline/group-resize/compute-resize.ts` + `rightPushChain` from `resize-controller.ts`.
- Timeline filmstrip: video elements render real per-position thumbnails, density adapts to zoom (power-of-two interval snapping). New `services/thumbnail-cache/` (LRU, batched mediabunny decode) + `timeline/components/video-filmstrip.tsx`. Images unchanged. Verified in browser by agent (distinct tiles, zoom density, viewport clipping, cache clear on media delete).
- Export hardening (`services/renderer/scene-exporter.ts`, `export-encoding.ts`): bitrate is fps-aware (`w*h*fps*bpp`, high = 0.2 bpp → ~12.4 Mbps at 1080p30, was ~6 fps-blind); keyframe every 2s; export canvas rounded to even dims. A capture-timing change (wait past WebGPU present before snapshot) was tried and REVERTED the same day — the canvas is blank after present, export came out black. Synchronous same-task snapshot is the correct behavior for WebGPU canvases; the temporal-ghosting hypothesis is dead. Colorspace tagging: no API exists (verified mediabunny + lib.dom) — BT.709 vs 601 is Chrome's encoder's call.
- MP4 export: ffmpeg.wasm + libx264 was implemented (chunked rawvideo → x264 CRF ladder) and then REVERTED at the owner's request after two wasm-side bugs (x264 auto-threading spawns 28 pthreads and hangs the worker — `-threads 4` fixes it; `@ffmpeg/ffmpeg` `writeFile` transfers/detaches the input ArrayBuffer, so chunk buffers must be single-use). The investigation notes stay here in case the idea is revisited; WebM/VP9 export is the owner's daily path and works well. MP4 stays on the mediabunny/WebCodecs avc path (which undershoots bitrate on this machine — OpenH264 ignores bitrateMode, has no quantizer mode, no hardware encoder exists; probe-verified 2026-07-28).
- Encoder investigation (2026-07-28, probe-verified in the user's Helium): NO hardware H.264 encode exists on the machine; Chrome's OpenH264 ignores `bitrateMode` (byte-identical VBR/CBR output), has no AVC quantizer mode (`isConfigSupported` false), and undershoots to ~0.6 Mbps on real content regardless of configured bitrate (6-50 Mbps). WebCodecs H.264 is a dead end for quality export; x264 via wasm is the only in-browser path. The fps-aware `computeVideoBitrate` still serves the WebM path.
- Next dev overlay disabled (`devIndicators: false` in `apps/web/next.config.ts`).
- Gates green: bun test 316 pass / 1 skip, tsc, eslint on touched files, `bun build:web`.
- Human verification still owed: freeze + ripple-extend click-through by owner.

## Done (2026-07-26/27 session)

- Freeze frame at playhead (snowflake button), composite canvas capture.
- Freeze frame is a single undo step (`BatchCommand` + `ShiftSplitRemainderCommand`).
- Freeze-duration prompt (persisted last-used, cancel-safe; `actions/freeze-frame-store.ts`).
- `MediaPreview` no longer crashes on image assets without `url` (thumbnail → placeholder fallback).
- Test suite runs on NixOS/bun 1.3 via `bunfig.toml` preload mock of `opencut-wasm`.
- Root `justfile` wraps common commands; `just` is in the nix devshell.
- Landing page stripped (`/` → `/projects`); Discord links + feedback UI removed.
- Sounds/stickers/effects UI gated behind `src/features.ts` flags (all off; code kept, entry points hidden).
- Project versioning v1 (`versions/`): named checkpoints + auto-checkpoints, restore with safety checkpoint, media pinning, versions popover. See notes/project-versioning.md.
- Single-file `.ocp` import/export (`project/transfer/`): zip with project.json + media + manifest, migrates older archives, full round-trip tested.
- Voice-over recording (`voiceover/`): mic → audio asset → timeline at playhead, one undo step.

Pending verification: freeze + single Ctrl+Z is unit-tested but never
smoke-tested by hand in a running browser. Same for the versions popover
and the freeze dialog — unit/coverage exists, a human click-through is
still owed.

## GPU vs CPU: what actually runs where (as of 2026-07-26)

Investigated `apps/web/src/services/renderer/` + `core/managers/renderer-manager.ts`.

**GPU (wgpu/WebGPU, via the Rust `opencut-wasm` package):**

- Final compositing of EVERY frame, preview and export alike
  (`CanvasRenderer.render` → resolve → build frame descriptor → upload
  textures → wasm `renderFrame`). The preview canvas literally is the wasm
  compositor's canvas.
- Effect passes (WGSL shaders) via `gpuRenderer.applyEffect`.
- Mask feathering via `gpuRenderer.applyMaskFeather`.

**Hardware-accelerated (WebCodecs, platform-dependent, via mediabunny):**

- Video decode for playback/thumbnails.
- Video encode on export.

**CPU:**

- Scene graph resolve, layout, animation sampling.
- Text/sticker/graphic rasterization into 2d canvases (then uploaded as GPU
  textures).
- Audio mixing (`AudioBuffer`), waveform/thumbnail generation.

**No CPU compositing fallback exists.** Without WebGPU: a "degraded" banner
shows, effects and mask feather are silently skipped, and `initCompositor`
throws — the preview cannot render at all. This matters for the Linux/BSD
first-class goal: WebGPU on Linux is Chrome-only today (Firefox behind a
flag), and effectively absent on BSD. Browser-first-class on those platforms
would need a fallback compositor (Canvas2d or WebGL) — a significant piece of
work. The GPUI desktop app sidesteps it entirely (native GPU).

## Roadmap (priority order, owner's call)

1. ~~Strip the landing page~~ done.
2. ~~Strip upstream-branded links~~ done.
3. ~~Disable sounds, stickers, effects~~ done behind `FEATURES` flags.
   Next step here: actually DELETE the flagged-off domain code (and the
   freesound API route, sticker providers, sounds panel internals) once the
   flags have baked a while — that also kills a chunk of the ~112
   pre-existing lint errors.
4. ~~Project import/export as one self-contained file~~ done (`.ocp`).
5. ~~Voice-over recording~~ done.
6. ~~Git-like project versioning~~ done (v1). v2 candidates: diff view,
   branches, content-addressed media dedup, shipping history inside `.ocp`.
7. ~~Self-contained binary, first-class Linux + BSD.~~ **GREENLIT
   2026-07-28: full 1:1 GPUI port, Linux-first.** Decisions: in-process
   ffmpeg (`ffmpeg-next`) for decode+encode, x264 for H.264 (licensing
   waived, quality first), no server backend (project store moves to
   filesystem), web version stays. BSD parked (GPUI unproven there).
   Full plan: notes/gpui-port.md.

## Deprioritized / parked

- Default keybinding for freeze frame — not wanted; the shortcuts UI already
  lets users bind it. A fuller "dynamic keybinding" feature is mostly
  already upstream (persisted keybindings store + shortcuts UI).
- Per-clip (non-composite) freeze capture — owner unconvinced; parked.
  Would need a single-element render pass in the GPU renderer.

## Quality gates (hardened 2026-07-26)

Baseline is GREEN: `bun test` (221 pass, 1 skip), `bunx tsc --noEmit`,
`bun build:web` all pass locally and in CI (the CI test step was a no-op
stub; it now runs typecheck + tests). `just verify` runs the full gate.
Rule from here: red means the agent broke it — no tolerated failures.
Full lint remains red (~112 pre-existing unsafe-assertion errors);
lint files you touch, full lint-green after the stripping pass.

Fixed along the way (were real runtime bugs, not just test issues):
`isActionWithOptionalArgs`/`isShortcutKey` guards restored (custom
keybindings crashed on load), stale positional `IndexedDBAdapter` /
`stickersRegistry.register` call sites updated (silent no-ops at runtime).

Remaining known breakage:

- One text-mask snap test is `test.skip` — needs a canvas 2d context bun
  does not provide (DOM-dependent).
- `apps/web/drizzle.config.ts` points at the stale schema path
  `./src/lib/db/schema.ts` (real one: `src/db/schema.ts`).

## Vision principles (owner's words, condensed)

- Unix philosophy: do one thing well. Open the editor, do your stuff, move on.
- Offline-first, privacy-first: projects and media stay local (IndexedDB).
- First-class Linux and BSD.
- Keep the browser editor — editing projects in a browser is a feature.
