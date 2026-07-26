import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import type { MediaType } from "@/media/types";
import { isRecord } from "@/services/storage/migrations/transformers/utils";

/**
 * Single-file project container (`.ocp`) — a zip archive holding:
 *
 * - `project.json` — the SerializedProject document exactly as the storage
 *   layer persists it (including its `version` number).
 * - `media/manifest.json` — metadata needed to rebuild every media File
 *   byte-identically (original name, MIME type, lastModified) plus the
 *   media-store metadata (dimensions, duration, thumbnail, …).
 * - `media/<assetId>-<sanitized name>` — one entry per media asset holding
 *   the original File bytes (stored, never re-encoded).
 */

export const PROJECT_ARCHIVE_EXTENSION = ".ocp";
export const PROJECT_JSON_ENTRY = "project.json";
export const MEDIA_MANIFEST_ENTRY = "media/manifest.json";
export const MEDIA_ENTRY_PREFIX = "media/";

export class ProjectArchiveError extends Error {
	constructor({ message }: { message: string }) {
		super(message);
		this.name = "ProjectArchiveError";
	}
}

export interface ArchivedMediaMetadata {
	id: string;
	name: string;
	mediaType: MediaType;
	fileType: string;
	lastModified: number;
	width?: number;
	height?: number;
	duration?: number;
	ephemeral?: boolean;
	thumbnailUrl?: string;
}

export interface ArchivedMediaEntry extends ArchivedMediaMetadata {
	entry: string;
}

export interface ArchiveMediaInput {
	metadata: ArchivedMediaMetadata;
	data: Uint8Array;
}

export interface ParsedProjectArchive {
	project: Record<string, unknown>;
	media: ArchivedMediaEntry[];
	readMediaData: ({ entry }: { entry: string }) => Uint8Array;
}

const MEDIA_MANIFEST_VERSION = 1;

export function sanitizeFileName({ name }: { name: string }): string {
	const sanitized = name
		.trim()
		.replace(/[^\w.-]+/g, "_")
		.replace(/^\.+/, "");
	return sanitized.length > 0 ? sanitized : "file";
}

export function buildMediaEntryName({
	assetId,
	name,
}: {
	assetId: string;
	name: string;
}): string {
	return `${MEDIA_ENTRY_PREFIX}${assetId}-${sanitizeFileName({ name })}`;
}

export function buildProjectArchive({
	projectJson,
	media,
}: {
	projectJson: string;
	media: ArchiveMediaInput[];
}): Uint8Array {
	const manifest: ArchivedMediaEntry[] = media.map(({ metadata }) => ({
		...metadata,
		entry: buildMediaEntryName({
			assetId: metadata.id,
			name: metadata.name,
		}),
	}));

	const files: Record<string, [Uint8Array, { level: 0 }]> = {
		[PROJECT_JSON_ENTRY]: [strToU8(projectJson), { level: 0 }],
		[MEDIA_MANIFEST_ENTRY]: [
			strToU8(JSON.stringify({ version: MEDIA_MANIFEST_VERSION, media: manifest })),
			{ level: 0 },
		],
	};
	for (const [index, { data }] of media.entries()) {
		const entry = manifest.at(index)?.entry;
		if (!entry) continue;
		files[entry] = [data, { level: 0 }];
	}

	return zipSync(files);
}

export function parseProjectArchive({
	archive,
}: {
	archive: Uint8Array;
}): ParsedProjectArchive {
	let files: Record<string, Uint8Array>;
	try {
		files = unzipSync(archive);
	} catch {
		throw new ProjectArchiveError({
			message: "Not a valid project archive (unreadable zip file)",
		});
	}

	const projectBytes = files[PROJECT_JSON_ENTRY];
	if (!projectBytes) {
		throw new ProjectArchiveError({
			message: "Not a valid project archive (project.json is missing)",
		});
	}

	let project: unknown;
	try {
		project = JSON.parse(strFromU8(projectBytes));
	} catch {
		throw new ProjectArchiveError({
			message: "Not a valid project archive (project.json is not valid JSON)",
		});
	}
	if (!isRecord(project) || Array.isArray(project)) {
		throw new ProjectArchiveError({
			message: "Not a valid project archive (project.json is not a project)",
		});
	}

	const media = parseMediaManifest({ files });

	return {
		project,
		media,
		readMediaData: ({ entry }: { entry: string }) => {
			const data = files[entry];
			if (!data) {
				throw new ProjectArchiveError({
					message: `Media file is missing from the archive (${entry})`,
				});
			}
			return data;
		},
	};
}

function parseMediaManifest({
	files,
}: {
	files: Record<string, Uint8Array>;
}): ArchivedMediaEntry[] {
	const manifestBytes = files[MEDIA_MANIFEST_ENTRY];
	if (!manifestBytes) {
		const hasMediaEntries = Object.keys(files).some((name) =>
			name.startsWith(MEDIA_ENTRY_PREFIX),
		);
		if (hasMediaEntries) {
			throw new ProjectArchiveError({
				message: "Not a valid project archive (media manifest is missing)",
			});
		}
		return [];
	}

	let parsed: unknown;
	try {
		parsed = JSON.parse(strFromU8(manifestBytes));
	} catch {
		throw new ProjectArchiveError({
			message: "Not a valid project archive (media manifest is not valid JSON)",
		});
	}
	if (!isRecord(parsed) || Array.isArray(parsed) || !Array.isArray(parsed.media)) {
		throw new ProjectArchiveError({
			message: "Not a valid project archive (media manifest is malformed)",
		});
	}

	return parsed.media.map((value) => parseManifestEntry({ value, files }));
}

function parseManifestEntry({
	value,
	files,
}: {
	value: unknown;
	files: Record<string, Uint8Array>;
}): ArchivedMediaEntry {
	const invalid = (reason: string): ProjectArchiveError =>
		new ProjectArchiveError({
			message: `Not a valid project archive (media manifest entry ${reason})`,
		});

	if (!isRecord(value) || Array.isArray(value)) throw invalid("is malformed");
	const { id, name, entry, mediaType, fileType, lastModified } = value;
	if (typeof id !== "string" || id.length === 0) throw invalid("has no id");
	if (typeof name !== "string") throw invalid(`(${id}) has no name`);
	if (
		typeof entry !== "string" ||
		!entry.startsWith(MEDIA_ENTRY_PREFIX) ||
		entry.includes("..")
	) {
		throw invalid(`(${id}) has an invalid file path`);
	}
	if (mediaType !== "image" && mediaType !== "video" && mediaType !== "audio") {
		throw invalid(`(${id}) has an invalid media type`);
	}
	if (typeof fileType !== "string") throw invalid(`(${id}) has no file type`);
	if (typeof lastModified !== "number" || !Number.isFinite(lastModified)) {
		throw invalid(`(${id}) has no last-modified timestamp`);
	}
	if (!files[entry]) {
		throw new ProjectArchiveError({
			message: `Not a valid project archive (media file ${entry} is missing)`,
		});
	}

	const { width, height, duration, ephemeral, thumbnailUrl } = value;
	return {
		id,
		name,
		entry,
		mediaType,
		fileType,
		lastModified,
		...(typeof width === "number" && { width }),
		...(typeof height === "number" && { height }),
		...(typeof duration === "number" && { duration }),
		...(typeof ephemeral === "boolean" && { ephemeral }),
		...(typeof thumbnailUrl === "string" && { thumbnailUrl }),
	};
}
