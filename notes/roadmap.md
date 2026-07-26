# Roadmap & session notes

Working notes for the fork's development direction. Captured 2026-07-26
(last commit `2f5ff9fe`; everything below "Done" is committed).

## Done

- Freeze frame at playhead (snowflake button), composite canvas capture.
- Freeze frame is a single undo step (`BatchCommand` + `ShiftSplitRemainderCommand`).
- `MediaPreview` no longer crashes on image assets without `url` (thumbnail → placeholder fallback).
- Test suite runs on NixOS/bun 1.3 via `bunfig.toml` preload mock of `opencut-wasm`.
- Root `justfile` wraps common commands; `just` is in the nix devshell.

Pending verification: freeze + single Ctrl+Z is unit-tested but never
smoke-tested by hand in a running browser.

## Tomorrow: freeze-duration prompt

Replace the hardcoded 3 s (`mediaTimeFromSeconds({ seconds: 3 })` in the
`freeze-frame` handler, `apps/web/src/actions/use-editor-actions.ts`) with a
small dialog asking for the duration before capture. Notes:

- Dialog components live in `apps/web/src/components/ui/` (shadcn-style radix).
- Keep the value a `MediaTime` (`mediaTimeFromSeconds`); validate > 0.
- Nice touch: remember the last-used duration in a zustand UI store
  (`*-store.ts`, UI/preference state only — never document state).
- The action is invoked from the toolbar button and (if bound) the shortcuts
  UI; the prompt should appear for both paths, so put it in the handler, not
  the toolbar.

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

1. **Strip the landing page.** App should open straight into the editor /
   project list. The marketing front page is dead weight.
2. **Strip upstream-branded links**: Discord link, "Send feedback" button.
3. **Disable (or remove) sounds, stickers, effects** — unfinished or buggy
   upstream. Prefer disabling behind a flag/registry skip first; delete once
   proven unnecessary. Effects are GPU shader passes + a registry
   (`registerDefaultEffects`); stickers and sounds each live in their own
   domain folders, so excision is mechanically easy.
4. **Project import/export as one self-contained file.** Project JSON +
   all media bundled in a single archive. Storage layer
   (`services/storage/`) already serializes projects (versioned migrations);
   media lives in IndexedDB as `File`s, so export = project JSON + media
   blobs into a zip; import = reverse through the same migration pipeline.
5. **Voice-over recording.** `MediaRecorder` (mic) → audio `MediaAsset` →
   drop on timeline at playhead. The audio pipeline (assets, tracks,
   waveform) already exists; this is mostly UI + recording plumbing.
6. **Git-like project versioning.** Named snapshots + history per project.
   The save manager and versioned storage are the natural foundation;
   snapshot blobs could reuse the export format from (4). Hardest and most
   novel item — design note needed before code.
7. **Self-contained binary, first-class Linux + BSD.** `apps/desktop` (GPUI)
   is the vehicle but is a hello-world scaffold today; porting the editor is
   a large effort. Caveat: GPUI's Linux support is young; BSD support is
   unproven — spike before committing. Browser version stays regardless
   (editing in a browser is a feature, not a compromise).

## Deprioritized / parked

- Default keybinding for freeze frame — not wanted; the shortcuts UI already
  lets users bind it. A fuller "dynamic keybinding" feature is mostly
  already upstream (persisted keybindings store + shortcuts UI).
- Per-clip (non-composite) freeze capture — owner unconvinced; parked.
  Would need a single-element render pass in the GPU renderer.

## Known pre-existing breakage (cleanup someday)

- `keybindings/__tests__/persistence.test.ts` imports
  `isActionWithOptionalArgs`, which does not exist anywhere — dead test file.
- Mask tests need a canvas 2d context bun doesn't provide (DOM-dependent).
- `custom mask point insertion` test asserts zero handles; code computes
  ±0.1 — stale test or stale code, never ran in CI to catch it.
- `resolve.test.ts` passes fractional ticks into the validated `mediaTime()`
  constructor — test bug.
- `eslint/rules/__tests__/prefer-object-params.test.mjs` uses `.only`,
  blocked when `CI` is set.
- `apps/web/drizzle.config.ts` points at the stale schema path
  `./src/lib/db/schema.ts` (real one: `src/db/schema.ts`).

## Vision principles (owner's words, condensed)

- Unix philosophy: do one thing well. Open the editor, do your stuff, move on.
- Offline-first, privacy-first: projects and media stay local (IndexedDB).
- First-class Linux and BSD.
- Keep the browser editor — editing projects in a browser is a feature.
