# Contributing to OpenCut

Thank you for your interest in contributing to OpenCut! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

- [Bun](https://bun.sh/docs/installation)
- Rust toolchain (only needed when editing `rust/`)

No env files, no database, no Docker — the editor is fully client-side; all state lives in IndexedDB.

### Setup

1. Fork the repository
2. Clone your fork locally
3. Install dependencies: `bun install`
4. Start the development server from the repo root: `bun dev:web`

The editor is at [http://localhost:3000](http://localhost:3000).

> **Note:** Web development uses the published `opencut-wasm` package by default, so a fresh clone does not need a local WASM build.
>
> If you are editing `rust/wasm`, run `bun run build:wasm`, then `cd rust/wasm/pkg && bun link`, then `cd ../../../apps/web && bun link opencut-wasm`.

### Desktop

Only needed if you're working on `apps/desktop` (Electron shell):

```bash
bun build:web                          # static export -> apps/web/out
bun run --cwd apps/desktop copy:web    # bundle it into the desktop app
bun run --cwd apps/desktop dev         # run the shell
```

`bun run --cwd apps/desktop dist` builds AppImage, `.deb`, and `.rpm` into `apps/desktop/release/`.

## How to Contribute

### Reporting Bugs

- Use the bug report template
- Include steps to reproduce
- Provide screenshots if applicable

### Suggesting Features

- Use the feature request template
- Explain the use case
- Consider implementation details

### Code Contributions

1. Create a new branch: `git checkout -b feature/your-feature-name`
2. Make your changes
3. Run the relevant checks for the area you touched:

   - Web changes: from `apps/web`, run `bun run lint` and `bun run format`; from the repo root, `bun test` and `bunx tsc --noEmit --project apps/web`
   - Desktop changes: `bun run --cwd apps/desktop build`

4. Commit your changes with a descriptive message
5. Push to your fork and create a pull request

## Code Style

- We use ESLint for linting and Prettier for formatting (tabs)
- Run `bun run format` from the `apps/web` directory to format code
- Run `bun run lint` from the `apps/web` directory to check for linting issues
- Follow the existing code patterns

## Pull Request Process

1. Fill out the pull request template completely
2. Link any related issues
3. Ensure CI passes
4. Request review from maintainers
5. Address any feedback

## Community

- Be respectful and inclusive
- Follow our Code of Conduct
- Help others in discussions and issues

Thank you for contributing!
