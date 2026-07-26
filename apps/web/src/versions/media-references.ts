import type { SerializedProject } from "@/services/storage/types";

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}

function collectFromElements({
	elements,
	mediaIds,
}: {
	elements: unknown;
	mediaIds: Set<string>;
}): void {
	if (!Array.isArray(elements)) return;

	for (const element of elements) {
		if (!isRecord(element)) continue;
		if (typeof element.mediaId === "string" && element.mediaId.length > 0) {
			mediaIds.add(element.mediaId);
		}
	}
}

/**
 * Extracts every media asset id referenced by a serialized project's scenes.
 * Duck-typed (like migrations) so snapshots of any project version scan safely.
 */
export function getReferencedMediaIds({
	project,
}: {
	project: SerializedProject;
}): Set<string> {
	const mediaIds = new Set<string>();

	for (const scene of project.scenes ?? []) {
		const tracks: unknown = scene.tracks;
		if (!isRecord(tracks)) continue;

		if (isRecord(tracks.main)) {
			collectFromElements({ elements: tracks.main.elements, mediaIds });
		}
		for (const list of [tracks.overlay, tracks.audio]) {
			if (!Array.isArray(list)) continue;
			for (const track of list) {
				if (!isRecord(track)) continue;
				collectFromElements({ elements: track.elements, mediaIds });
			}
		}
	}

	return mediaIds;
}
