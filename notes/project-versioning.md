# Project versioning (git-like) — design

Status: concept, agreed 2026-07-26. Implementation not started.

## Goal

Named, known-good versions of a project ("fertige Versionen") that you can
jump between freely and losslessly, plus automatic checkpoints so work is
never lost — without leaving the offline-first, IndexedDB-only model.

Owner decisions (locked):

- **Linear history + named checkpoints**, no real branches in v1.
- **Both triggers**: manual named snapshots + auto-checkpoints.
- **Retention**: auto-checkpoints rotate (ring buffer), named versions are
  kept forever.
- **v1 is jump-only** — no diff view.

## Model

One linear history per project. Entries:

```ts
type ProjectVersion = {
	id: string;
	projectId: string;
	kind: "named" | "auto";
	name?: string; // required for kind "named"
	createdAt: Date;
	project: SerializedProject; // same shape the storage layer already writes
	mediaHashes: string[]; // media referenced by this snapshot
	thumbnail?: string; // data URL, via renderer.captureFrame()
};
```

A project always has a **head** (the live document). Jumping = point head at
an older version and continue from there; the abandoned "future" is not
deleted — a safety auto-checkpoint is written before every jump, so every
position stays reachable.

### Auto-checkpoints

- Written on save (hook in `SaveManager`) — throttled (e.g. min 60 s apart)
  so rapid autosaves don't spam the ring.
- Ring buffer: keep the newest **20** auto versions per project; older auto
  versions are deleted on write. Named versions never rotate.

### Media storage

Snapshots share media, they don't copy it. Media blobs move to a
**content-addressed store**: `mediaBlobs: { hash, blob }` with
`MediaAsset` records referencing hashes. A blob is collected only when no
project document and no version snapshot references it. This keeps 50
snapshots of a project with one 200 MB video at ~one video's cost.

## Storage layer

New versioned migration (next schema version after current v31) adding two
stores: `projectVersions` and `mediaBlobs`. Follow the existing per-version
migration pattern and its 18-test fixture suite
(`services/storage/migrations/__tests__/`). Existing per-project media
records get re-pointed at hashed blobs by the migration.

## Surfaces (all via the actions system, per repo conventions)

- `create-checkpoint` — prompt for a name, capture thumbnail
  (`renderer.captureFrame()` already exists from freeze frame), write named
  version. Toolbar button + bindable action.
- `restore-version` (from the list UI) — writes a safety auto-checkpoint,
  swaps head, reloads the editor document.
- `rename-version`, `delete-version` (named only).
- Version list UI: popover/panel in the editor — name, date, kind badge,
  thumbnail; click to jump.

Restore is a wholesale document swap, not a `Command` — it deliberately sits
outside undo/redo (undo of a restore = the safety checkpoint written before
it).

## Explicitly out of scope (v1)

- Branches/forks of history.
- Diff view between versions.
- Versioning of media assets themselves (asset edits replace blobs).
- Shipping history inside the single-file export (natural later extension:
  export current head only, optionally with history).

## Open implementation questions

- Exact retention numbers (20 auto / 60 s throttle are defaults, tune).
- Quota handling when a snapshot write exceeds IndexedDB quota (surface the
  existing quota-exceeded toast path).
- Whether auto-checkpoints pause during playback (probably yes — save is
  already throttled there; check `SaveManager`).

## Test plan

- Migration v→v+1 fixture test (copy the existing pattern).
- Rotation: 21st auto-checkpoint evicts the oldest auto, never a named one.
- Restore round-trip: snapshot → mutate → restore → deep-equal document.
- Safety checkpoint on jump: jumping from a dirty head preserves it.
- Media GC: blob survives while any version references it; collected when
  none do.
- Throttle: two saves inside the window produce one auto-checkpoint.
