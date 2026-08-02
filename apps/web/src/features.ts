/**
 * Feature flags for upstream domains that are unfinished or buggy.
 * Flags hide UI entry points ONLY — document data and rendering of existing
 * projects must stay untouched, so old projects keep working. Deletion of
 * the dead code comes later, once the flags have proven themselves.
 */
export const FEATURES = {
	sounds: false,
	stickers: false,
	effects: true,
} as const;
