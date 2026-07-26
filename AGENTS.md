# Repository Guidelines

## Project Overview

Personal fork of [opencut-app/opencut-classic](https://github.com/opencut-app/opencut-classic) — the archived "OpenCut Legacy" codebase deployed at opencut.app. A privacy-first, CapCut-style video editor that runs in the browser; projects and media live in IndexedDB, so the editor works fully offline of any backend. We do not track or contribute to upstream; extend freely. The clone started as `--depth 1` of upstream `main` (May 2026); run `git fetch --unshallow` if full history is ever needed, and point `origin` at your own remote for pushes.

## Architecture & Data Flow

Bun + Turborepo monorepo with three pillars:

1. `apps/web` — Next.js 16 (App Router, Turbopack dev) React 19 editor. Consumes the Rust core as the npm dependency `opencut-wasm`.
2. `rust/` — Cargo workspace (root `Cargo.toml`) of 6 shared crates (`time`, `bridge`, `effects`, `gpu`, `masks`, `compositor`; wgpu + WGSL shaders) plus `rust/wasm`, the cdylib compiled by wasm-pack into `rust/wasm/pkg` and published as `opencut-wasm`. Business logic is actively migrating from TypeScript to here.
3. `apps/desktop` — GPUI 0.2.2 native app, currently a hello-world scaffold window. Ignore unless explicitly working on it.

The web app has a hybrid state architecture:

- `EditorCore` (`apps/web/src/core/index.ts`) is a singleton wiring 12 manager classes (`timeline`, `command`, `playback`, `scenes`, `project`, `media`, `renderer`, `save`, `audio`, `selection`, `clipboard`, `diagnostics` in `core/managers/`). Managers own ALL document/editor state and expose subscribe/notify.
- React reads manager state via `useEditor(selector)` (`apps/web/src/editor/use-editor.ts`, built on `useSyncExternalStore`). zustand v5 is used ONLY for UI/preference state, in colocated `*-store.ts` files.
- Every document mutation is a `Command` subclass (`apps/web/src/commands/**`, `base-command.ts`: `execute(): CommandResult|undefined; undo(); redo()`), built by a manager and run through `CommandManager` (`core/managers/commands.ts`), which maintains undo/redo history, selection snapshots, post-command reactors (e.g. pruning empty tracks), and ripple-edit adjustments.
- User input flows through the actions system: `actions/definitions.ts` catalogs typed action ids + default shortcuts → keybindings dispatch `invokeAction(id, args)` (`actions/registry.ts`) → handlers in `actions/use-editor-actions.ts` call `editor.*` managers (bound via `useActionHandler` inside `EditorProvider`).

Typical flow (split at playhead, key "s"): keybindings-store → `invokeAction("split")` → handler reads `editor.playback.getCurrentTime()` + active scene tracks → `editor.timeline.splitElements({elements, splitTime})` (`core/managers/timeline-manager.ts`) → builds `SplitElementsCommand` (`commands/timeline/element/split-elements.ts`) → `command.execute()` mutates tracks and snapshots for undo → CommandManager pushes history + runs reactors → managers notify → UI re-renders via `useEditor` selectors.

## Key Directories

| Path | Purpose |
|---|---|
| `apps/web/src/core/` | `EditorCore` singleton + the 12 managers (source of truth for document state) |
| `apps/web/src/actions/` | Action catalog, registry/dispatch, handler implementations, keybindings store |
| `apps/web/src/commands/` | Undoable `Command` classes grouped by domain (`timeline/`, `scene/`, `project/`, `media/`) |
| `apps/web/src/timeline/` | Timeline domain: components, hooks, placement logic, `timeline-store.ts` (UI state only) |
| `apps/web/src/media/` | Media processing, thumbnails, upload, mediabunny wrappers |
| `apps/web/src/services/` | `renderer/` (GPU), `storage/` (IndexedDB + versioned migrations), `transcription/`, caches |
| `apps/web/src/wasm/` | `@/wasm` wrapper over `opencut-wasm` — only `wasm/media-time.ts` imports the package directly |
| `apps/web/src/components/` | Shared React: `ui/` (shadcn-style radix), `editor/`, `providers/` |
| `apps/web/src/app/` | App Router pages + `api/` routes |
| `rust/crates/` | Shared Rust crates; `rust/wasm/` compiles them to the `opencut-wasm` npm package |
| `docs/` | Subsystem docs: `actions.md`, `keyframes.md`, `effects-renderer.md`, `countries-search.md` |
| `notes/` | Design notes (e.g. `primitives-vs-domains.md`) |

Domain folders also include `animation/`, `effects/`, `masks/`, `params/`, `preview/`, `retime/`, `ripple/`, `selection/`, `stickers/`, `subtitles/`, `text/` — each self-contained with its own registry/store/resolver.

## Development Commands

Run from repo root unless noted. `flake.nix` provides a devshell (`nix develop`) with bun (1.3.x) + nodejs (24); the pinned bun 1.2.18 works but so does 1.3.x.

```bash
bun install
cp apps/web/.env.example apps/web/.env.local   # required; zod validates env at import time
docker compose up -d db redis serverless-redis-http   # optional; not needed for editor work
bun dev:web        # http://localhost:3000 (editor at /editor/<project_id>)
bun build:web      # next build via turbo
bun test           # all JS/TS tests (Bun test runner)
bun lint:web       # eslint apps/web/src
bun lint:web:fix   # with --fix
```

Inside `apps/web`: `bun run dev`, `bun run format` (prettier all of `src`), `bun run db:migrate` / `db:generate` (drizzle-kit), `bun run preview` / `deploy` (OpenNext → Cloudflare Workers). Typecheck with `bunx tsc --noEmit` (no script exists; see Testing & QA for pre-existing errors).

WASM (only when editing `rust/`): `./script/setup-rust` installs rustup + wasm-pack; then `bun build:wasm` (wasm-pack → `rust/wasm/pkg`), `bun dev:wasm` (cargo-watch rebuild), and `bun link` in `rust/wasm/pkg` + `bun link opencut-wasm` in `apps/web` to use the local build (`bun add opencut-wasm` to revert). Rust tests: `cargo test -p <crate>`. Desktop: `cargo run -p opencut-desktop` after `apps/desktop/script/setup`.

Self-hosting: `docker compose up -d` (full stack at http://localhost:3100; postgres 17, redis 7, serverless-redis-http Upstash shim on :8079).

## Code Conventions & Common Patterns

- Formatting: Prettier is the formatter of record (`useTabs: true`). Linting: ESLint 9 flat config (`eslint.config.mjs`) scoped to `apps/web/src/**/*.{ts,tsx}` — type-aware, stacks js/tseslint/react/react-hooks/jsx-a11y/next core-web-vitals with `eslint-config-prettier` last. `biome.json` is vestigial upstream config; no script or dep uses it — do not reach for biome.
- Custom rule `opencut/prefer-object-params` (`eslint/rules/prefer-object-params.mjs`, error): functions take a single destructured object param (`splitElements({elements, splitTime})`), not multiple positional params. Direct callbacks are exempt.
- Naming: zustand stores are `*-store.ts` exporting `useXStore`, created with `create<T>()(persist(...))`. Never put document state in zustand — use EditorCore managers (timeline-store.ts header says exactly this).
- Never mutate scene/track/element state directly; go through a manager method that builds a `Command` and runs `editor.command.execute({command})`, so the change is undoable. New user-facing operations should be registered in `actions/definitions.ts` + implemented in `use-editor-actions.ts` (see `docs/actions.md`).
- Time is a branded ticks type: `MediaTime = number & { __mediaTime: unique symbol }` from `@/wasm`. Use `mediaTimeFromSeconds`, `addMediaTime`, `clampMediaTime`, etc. — never raw arithmetic on stored times. Constructors take object params and validate (non-integer ticks throw).
- Async: managers expose Promise-returning methods (e.g. `renderer.captureFrame(): Promise<Blob | null>`); commands themselves are sync. Media processing goes through mediabunny wrappers in `src/media/`.
- When creating image assets programmatically, set BOTH `url` (`URL.createObjectURL(file)`) and `thumbnailUrl` (`generateImageThumbnail` from `src/media/processing.ts`) — `MediaPreview` in `components/editor/panels/assets/views/assets.tsx` renders `item.url ?? ""` and crashes the assets panel on empty src.
- Env vars are zod-validated at import time (`apps/web/src/env/`); the dev server 500s without `.env.local`. Dummy values suffice for editor work — no real Postgres/Redis/Cloudflare needed.
- TS path alias `@/*` → `apps/web/src/*` (also `content-collections` → `./.content-collections/generated`). Always import wasm via `@/wasm`.

## Important Files

| File | Role |
|---|---|
| `apps/web/src/app/layout.tsx` | Root layout (ThemeProvider, TooltipProvider, Toaster) |
| `apps/web/src/app/editor/[project_id]/page.tsx` | Editor page — resizable panels inside `EditorProvider` |
| `apps/web/src/components/providers/editor-provider.tsx` | Loads project, initializes GPU renderer, mounts actions + keybindings |
| `apps/web/src/core/index.ts` | EditorCore singleton; wires all managers + command reactor |
| `apps/web/src/core/managers/commands.ts` | Undo/redo history, selection overrides, reactors, ripple |
| `apps/web/src/actions/definitions.ts` | Action catalog + default shortcuts (add new actions here) |
| `apps/web/src/actions/use-editor-actions.ts` | Action handler implementations |
| `apps/web/src/commands/base-command.ts` | `abstract class Command` contract |
| `apps/web/src/wasm/media-time.ts` | Branded MediaTime type + validated wasm wrappers |
| `apps/web/src/db/schema.ts` | Drizzle schema (note: `apps/web/drizzle.config.ts` points at the stale path `./src/lib/db/schema.ts`) |
| `eslint.config.mjs` | Flat ESLint config + inline `eslint-plugin-opencut` |
| `turbo.json` | Task graph (build/dev/preview/deploy/lint/format) |
| `apps/web/wrangler.jsonc` + `open-next.config.ts` | Cloudflare Workers deploy target via OpenNext |
| `docker-compose.yml`, `apps/web/Dockerfile` | Local services + self-hosted production image |

## Runtime/Tooling Preferences

- Bun is the package manager and script runner, pinned `1.2.18` via `packageManager` in root and `apps/web/package.json`; bun 1.3.x (from the Nix devshell) also works for install/dev/tsc. Lockfile: root `bun.lock` only. `.npmrc`: `install-strategy=nested`, `node-linker=isolated`.
- No `engines` field; Node 24 comes from the devshell but the app is built/run by bun and Next. CI pins bun 1.2.18.
- Rust toolchain (rustup, wasm-pack, cargo-watch) is needed ONLY for `rust/` or `apps/desktop` work — install via `script/setup-rust`. The web app normally uses the published `opencut-wasm` npm package; no local Rust required for editor work.
- Deploy target is Cloudflare Workers via `@opennextjs/cloudflare` (worker name `opencut`); self-hosting alternative is the Docker standalone build. Root `wrangler.jsonc` duplicates the apps/web one — edit `apps/web/wrangler.jsonc`.
- Dead config to ignore: root `build:tools`/`dev:tools`/`start:tools` scripts (target nonexistent `@opencut/tools`), turbo `check-types` task (no package implements it), root `format:web` (formats only `src/services/renderer`; use `bun run format` in apps/web), `packages/` workspace glob (empty).

## Testing & QA

- Framework: Bun's built-in test runner (`import from "bun:test"`), zero config. Run everything with `bun test` from the repo root (discovers across workspaces). Single file: `bun test <path>`.
- Root `bunfig.toml` registers `[test] preload` → `apps/web/src/test-utils/mock-opencut-wasm.ts`: the published `opencut-wasm` package is wasm-pack bundler-target glue whose `.wasm` ESM import bun's test runner does not instantiate, so a faithful pure-TS mock (mirrors `rust/crates/time`, TICKS_PER_SECOND=120_000) is registered before any test module evaluates. Extend its export surface when a test transitively links new `opencut-wasm` imports.
- ~30 test files, colocated in `__tests__/` dirs next to source: timeline placement/pipeline, retime, animation, fps, masks, params, keybindings, stickers, math. Deepest investment: `apps/web/src/services/storage/migrations/__tests__/` — 18 per-version-step tests (v0-to-v1 … v30-to-v31) with shared `helpers.ts` and `fixtures/` sample projects; copy this pattern when adding a storage migration.
- Conventions: pure-function unit tests, nested describe blocks, deterministic dates. No coverage config anywhere.
- CI (`.github/workflows/bun-ci.yml`, sole workflow) builds the WASM package and the Next app on ubuntu/windows/macos but its test step is a no-op stub (`echo "No tests implemented yet"`, continue-on-error); no lint, format, or typecheck gates. Tests run locally only.
- Rust crates have `#[cfg(test)]` unit tests (time crate, bridge) reachable only via `cargo test -p <crate>` — not wired to scripts or CI.
- `bunx tsc --noEmit` in `apps/web` reports ~27 lines of PRE-EXISTING upstream type errors (storage migrations v1-to-v2, stickers providers, placement tests). Only care about errors in files you touched.

## Fork Notes & Current Work

### Freeze frame (implemented, working)

The timeline toolbar snowflake button (upstream had it disabled with a "coming soon" tooltip) now freezes the frame at the playhead: it captures the composited canvas as PNG, adds it as an image media asset, splits the video element at the playhead, shifts the right part right by the freeze duration, and inserts a 3-second image element in the gap.

Files:

- `apps/web/src/actions/definitions.ts` — registered the `freeze-frame` action (category `editing`, no default keybinding; bindable in the shortcuts UI).
- `apps/web/src/actions/use-editor-actions.ts` — the `freeze-frame` handler: target resolution, capture, asset creation (sets `url` + `thumbnailUrl`, both required — see pitfalls), then a single `BatchCommand` (asset add, split, shift, insert) executed once.
- `apps/web/src/commands/timeline/element/shift-split-remainder.ts` — `ShiftSplitRemainderCommand`: moves the right-side elements a `SplitElementsCommand` produces; reads `getRightSideElements()` lazily at `execute()` so it can be composed in a `BatchCommand` after the split (and rebuilds on redo, which re-generates right-side ids).
- `apps/web/src/core/managers/renderer-manager.ts` — new public `captureFrame(): Promise<Blob | null>`; `createSnapshot()` (used by save/copy snapshot) was refactored to reuse it.
- `apps/web/src/timeline/components/timeline-toolbar.tsx` — button enabled, wired to the action via `handleAction`; `canFreezeFrame` selector disables it unless a video element is under the playhead.
- `apps/web/src/media/processing.ts` — `generateImageThumbnail` is now exported for reuse by the handler.

Behavior details and known limitations:

- Freeze duration is a fixed 3 seconds (`mediaTimeFromSeconds({ seconds: 3 })` in the handler); adjust by trimming the inserted element.
- The still is a composite capture of the whole canvas at the current time — overlays, text, and stickers visible at that moment are baked in. CapCut freezes only the clip itself; per-clip capture would need a single-element render pass.
- Target is the first selected video element under the playhead (or the first video element under the playhead if nothing is selected). With stacked videos on several tracks it may pick a non-topmost one.
- One freeze is a single undo step: the handler builds one `BatchCommand` (`AddMediaAssetCommand` + `SplitElementsCommand` + `ShiftSplitRemainderCommand` + `InsertElementCommand`) and executes it once via `editor.command.execute`.
- Freeze-frame assets survive reload: `storageService` recreates object URLs from the stored `File` on project load, and the thumbnail is a data URL.

### Pitfalls

- `MediaPreview` (`apps/web/src/components/editor/panels/assets/views/assets.tsx`) renders image assets with `item.url ?? item.thumbnailUrl`, falling back to `MediaTypePlaceholder` when neither exists. Still, when creating image assets programmatically, always set both `url` (`URL.createObjectURL(file)`) and `thumbnailUrl` (`generateImageThumbnail`) — the placeholder is a degraded fallback, not a presentation anyone wants.
- New timeline-affecting operations should go through the actions system (`definitions.ts` + `useActionHandler` in `use-editor-actions.ts`) and editor managers (`editor.timeline.*`, `editor.media.*`, `editor.renderer.*`), which wrap everything in undoable commands. MediaTime is a branded ticks type from `@/wasm` — use `mediaTimeFromSeconds`, `addMediaTime`, etc., never raw arithmetic on stored values.

### Open next steps

1. ~~Commit the freeze-frame work~~ done (`fc0a801e`); ~~single-command undo~~ done (`BatchCommand` + `ShiftSplitRemainderCommand`); ~~`MediaPreview` empty-`src` guard~~ done (thumbnail/placeholder fallback).
2. Optional: freeze-duration prompt, per-clip (non-composite) capture, default keybinding.
