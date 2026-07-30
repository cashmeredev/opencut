# Common commands for the OpenCut fork. Run inside the devshell (`nix-shell`)
# or anywhere bun + node are on PATH. `just` lists all recipes.

default:
    @just --list

# Install dependencies
install:
    bun install

# Create apps/web/.env.local from the example if missing (dev server 500s without it)
env:
    @test -f apps/web/.env.local || { cp apps/web/.env.example apps/web/.env.local && echo "Created apps/web/.env.local from example"; }

# First-time setup: install + env file
setup: install env
    @echo "Ready. Run 'just dev'."

# Start the web editor dev server (http://localhost:3000, editor at /editor/<project_id>)
dev: env
    bun dev:web

# Production build of the web app
build:
    bun build:web

# Run tests; pass a path for a single file: just test apps/web/src/timeline
test *args:
    bun test {{args}}

# Lint the web app
lint:
    bun lint:web

# Lint and auto-fix
lint-fix:
    bun lint:web:fix

# Typecheck the web app (pre-existing upstream errors exist; only care about files you touched)
typecheck:
    cd apps/web && bunx tsc --noEmit

# Format the web app source with Prettier
format:
    cd apps/web && bun run format

# Full local quality gate: tests, typecheck, production build.
# Full lint is NOT part of the gate yet (~112 pre-existing upstream errors);
# lint the files you touched with `just lint`.
verify:
    bun test
    cd apps/web && bunx tsc --noEmit
    bun build:web

# Build the Rust WASM package (requires rustup + wasm-pack)
wasm:
    bun build:wasm

# Rebuild the WASM package on Rust changes
wasm-watch:
    bun dev:wasm

# Run Rust tests for one crate: just rust-test time
rust-test crate:
    cargo test -p {{crate}}

# Run the Electron desktop app (requires apps/web/out via bun build:web + just desktop-web)
desktop:
    bun run --cwd apps/desktop dev

# Copy the static web bundle into the desktop app
desktop-web:
    bun run --cwd apps/desktop copy:web

# Start optional local services (postgres, redis) — not needed for editor work
services:
    docker compose up -d db redis serverless-redis-http

# Start the full self-hosted stack at http://localhost:3100
stack:
    docker compose up -d
