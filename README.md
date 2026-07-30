# OpenCut

Offline-first, privacy-first video editor. Your projects and media never leave your device.

Continues the abandoned [opencut-app/OpenCut](https://github.com/opencut-app/OpenCut) codebase: the editor was already fully client-side, so the dead backend (auth, database, blog) was ripped out. The web app is now a fully static site, and there is a self-contained Linux desktop app with native ffmpeg exports.

## Variants

- **Desktop (Linux)**: self-contained app — AppImage, `.deb`, `.rpm` on the [releases page](https://github.com/cashmeredev/opencut/releases). Exports are encoded by a bundled ffmpeg (libx264 / libvpx-vp9) instead of the browser encoder, so MP4 quality settings are actually honored and colors are correctly tagged BT.709.
- **Web**: fully static site, host it anywhere. Grab `opencut-web-static.tar.gz` from releases and serve it with any static file server.

## Project structure

- `apps/web/`: Next.js editor, built as a static export (`next build` → `out/`). All state lives in IndexedDB.
- `apps/desktop/`: Electron shell. Serves the static web build over a privileged `opencut://` protocol and bridges exports to a bundled ffmpeg over IPC.
- `rust/`: wgpu compositor, effects, masks, and time primitives compiled to WASM (`opencut-wasm`).

## Development

Prerequisites: [Bun](https://bun.sh/docs/installation). No env files, no database, no Docker.

```bash
bun install
bun dev:web
```

The editor is at [http://localhost:3000](http://localhost:3000).

### Desktop

```bash
bun build:web                          # static export -> apps/web/out
bun run --cwd apps/desktop copy:web    # bundle it into the desktop app
bun run --cwd apps/desktop dev         # run the shell
```

Package for Linux (AppImage, deb, rpm into `apps/desktop/release/`):

```bash
bun run --cwd apps/desktop dist
```

### Local WASM development

Only needed if you're editing `rust/wasm` and want the web app to use your local build instead of the published package. Requires a Rust toolchain, `wasm-pack`, and `cargo-watch`:

```bash
bun run build:wasm        # build once
cd rust/wasm/pkg && bun link
cd apps/web && bun link opencut-wasm
bun dev:wasm              # rebuild on changes
```

Switch back to the published package with `cd apps/web && bun add opencut-wasm`.

### Self-hosting the web app

```bash
docker compose up -d
```

Serves the static build at [http://localhost:3000](http://localhost:3000). Any static file server works — the build output is `apps/web/out/`.

## Releases

Tags (`v*`) trigger the GitHub release workflow: static web tarball plus AppImage, `.deb`, and `.rpm`.

## License

[MIT LICENSE](LICENSE)
