import type { ProjectRecord } from "@/services/storage/migrations/transformers/types";
import { isRecord } from "@/services/storage/migrations/transformers/utils";

/**
 * Pure document helpers for project transfer: walking the media references of
 * a serialized project, remapping them to fresh media-store ids on import,
 * and resolving imported project names.
 *
 * All functions tolerate malformed shapes (they operate on untrusted parsed
 * JSON) and never mutate their input.
 */

function collectTracks({ tracks }: { tracks: ProjectRecord }): unknown[] {
	const collected: unknown[] = [];
	if (isRecord(tracks.main)) {
		collected.push(tracks.main);
	}
	for (const key of ["overlay", "audio"] as const) {
		const group = tracks[key];
		if (Array.isArray(group)) {
			collected.push(...group);
		}
	}
	return collected;
}

function walkElements({
	project,
	visit,
}: {
	project: unknown;
	visit: ({ element }: { element: ProjectRecord }) => void;
}): void {
	if (!isRecord(project)) return;
	const scenes = project.scenes;
	if (!Array.isArray(scenes)) return;

	for (const scene of scenes) {
		if (!isRecord(scene)) continue;
		const tracks = scene.tracks;
		if (!isRecord(tracks)) continue;
		for (const track of collectTracks({ tracks })) {
			if (!isRecord(track)) continue;
			const elements = track.elements;
			if (!Array.isArray(elements)) continue;
			for (const element of elements) {
				if (isRecord(element)) {
					visit({ element });
				}
			}
		}
	}
}

export function collectReferencedMediaIds({
	project,
}: {
	project: unknown;
}): string[] {
	const ids = new Set<string>();
	walkElements({
		project,
		visit: ({ element }) => {
			const mediaId = element.mediaId;
			if (typeof mediaId === "string" && mediaId.length > 0) {
				ids.add(mediaId);
			}
		},
	});
	return [...ids];
}

export function remapProjectMediaIds({
	project,
	idMap,
}: {
	project: ProjectRecord;
	idMap: ReadonlyMap<string, string>;
}): ProjectRecord {
	if (idMap.size === 0) return project;

	const remapped = structuredClone(project);
	walkElements({
		project: remapped,
		visit: ({ element }) => {
			const mediaId = element.mediaId;
			if (typeof mediaId !== "string") return;
			const nextId = idMap.get(mediaId);
			if (nextId !== undefined) {
				element.mediaId = nextId;
			}
		},
	});
	return remapped;
}

export function resolveImportedProjectName({
	name,
	existingNames,
}: {
	name: string;
	existingNames: string[];
}): string {
	return existingNames.includes(name) ? `${name} (imported)` : name;
}
