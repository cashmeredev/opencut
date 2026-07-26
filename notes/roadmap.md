# Roadmap & session notes

Working notes for the fork's development direction. Captured 2026-07-26,
updated 2026-07-27 (roadmap feature slices implemented, last commit
`9c3b788b`).

## Done

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
7. **Self-contained binary, first-class Linux + BSD.** `apps/desktop` (GPUI)
   is the vehicle but is a hello-world scaffold today; porting the editor is
   a large effort. Caveat: GPUI's Linux support is young; BSD support is
   unproven — spike before committing. Browser version stays regardless
   (editing in a browser is a feature, not a compromise). Distribution:
   AppImage/GitHub releases first; Flathub is effectively closed to us (June
   2026 policy bans AI-assisted code in new submissions).

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
