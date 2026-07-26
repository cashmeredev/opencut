import type { TProject } from "@/project/types";
import type { MediaAsset } from "@/media/types";
import {
	CURRENT_PROJECT_VERSION,
	migrations,
} from "@/services/storage/migrations";
import {
	deserializeProject,
	serializeProject,
	storageService,
} from "@/services/storage/service";
import type { SerializedProject } from "@/services/storage/types";
import { isRecord } from "@/services/storage/migrations/transformers/utils";
import { generateUUID } from "@/utils/id";
import {
	type ArchiveMediaInput,
	PROJECT_ARCHIVE_EXTENSION,
	ProjectArchiveError,
	buildProjectArchive,
	parseProjectArchive,
	sanitizeFileName,
} from "./container";
import { getRecordVersion, migrateProjectRecord } from "./migrate";
import {
	collectReferencedMediaIds,
	remapProjectMediaIds,
	resolveImportedProjectName,
} from "./remap";

/**
 * Browser-facing project transfer: build a single-file `.ocp` archive from a
 * stored project (export) and restore an archive as a brand-new project
 * (import). Storage writes go through the same storage service path the app
 * already uses; imported documents older than the current version go through
 * the same migration registry as storage migrations.
 */

export interface ExportProjectArchiveResult {
	blob: Blob;
	fileName: string;
	missingMediaIds: string[];
}

export interface ImportProjectArchiveResult {
	project: TProject;
	importedMediaCount: number;
}

export async function exportProjectArchive({
	projectId,
}: {
	projectId: string;
}): Promise<ExportProjectArchiveResult> {
	const result = await storageService.loadProject({ id: projectId });
	if (!result) {
		throw new ProjectArchiveError({ message: "Project not found" });
	}

	const serialized = serializeProject({ project: result.project });
	const referencedMediaIds = collectReferencedMediaIds({
		project: serialized,
	});

	const assets = await storageService.loadAllMediaAssets({ projectId });
	const assetIds = new Set(assets.map((asset) => asset.id));
	const missingMediaIds = referencedMediaIds.filter((id) => !assetIds.has(id));

	const media: ArchiveMediaInput[] = [];
	for (const asset of assets) {
		media.push({
			metadata: toArchivedMediaMetadata({ asset }),
			data: new Uint8Array(await asset.file.arrayBuffer()),
		});
	}

	const archive = buildProjectArchive({
		projectJson: JSON.stringify(serialized),
		media,
	});

	return {
		blob: new Blob([new Uint8Array(archive)], { type: "application/zip" }),
		fileName: `${sanitizeFileName({ name: result.project.metadata.name })}${PROJECT_ARCHIVE_EXTENSION}`,
		missingMediaIds,
	};
}

function toArchivedMediaMetadata({
	asset,
}: {
	asset: MediaAsset;
}): ArchiveMediaInput["metadata"] {
	// Object-URL thumbnails only live for the exporting session; data-URL
	// thumbnails are the persisted form and survive the transfer.
	const thumbnailUrl = asset.thumbnailUrl?.startsWith("blob:")
		? undefined
		: asset.thumbnailUrl;

	return {
		id: asset.id,
		name: asset.name,
		mediaType: asset.type,
		fileType: asset.file.type,
		lastModified: asset.file.lastModified,
		...(asset.width !== undefined && { width: asset.width }),
		...(asset.height !== undefined && { height: asset.height }),
		...(asset.duration !== undefined && { duration: asset.duration }),
		...(asset.ephemeral !== undefined && { ephemeral: asset.ephemeral }),
		...(thumbnailUrl !== undefined && { thumbnailUrl }),
	};
}

export async function importProjectArchive({
	file,
	existingNames,
}: {
	file: File;
	existingNames: string[];
}): Promise<ImportProjectArchiveResult> {
	// Parse and validate everything before writing a single byte to storage,
	// so a bad archive can never leave a half-written project behind.
	const parsed = parseProjectArchive({
		archive: new Uint8Array(await file.arrayBuffer()),
	});

	const fromVersion = getRecordVersion({ project: parsed.project });
	if (fromVersion > CURRENT_PROJECT_VERSION) {
		throw new ProjectArchiveError({
			message: `This project was exported from a newer version (project v${fromVersion}, app supports v${CURRENT_PROJECT_VERSION})`,
		});
	}

	const migrated = await migrateProjectRecord({ project: parsed.project, migrations });
	if (!migrated.complete) {
		throw new ProjectArchiveError({
			message: `This project (v${fromVersion}) could not be migrated to the current version`,
		});
	}

	const idMap = new Map<string, string>();
	for (const entry of parsed.media) {
		idMap.set(entry.id, generateUUID());
	}
	const remapped = remapProjectMediaIds({
		project: migrated.project,
		idMap,
	});
	assertSerializedProject(remapped);

	const projectId = generateUUID();
	const serialized: SerializedProject = {
		...remapped,
		metadata: {
			...remapped.metadata,
			id: projectId,
			name: resolveImportedProjectName({
				name: remapped.metadata.name,
				existingNames,
			}),
		},
	};
	const project = deserializeProject({ serialized });

	try {
		for (const entry of parsed.media) {
			const assetId = idMap.get(entry.id);
			if (!assetId) {
				throw new ProjectArchiveError({
					message: `Media asset ${entry.id} could not be remapped`,
				});
			}
			const mediaFile = new File(
				[new Uint8Array(parsed.readMediaData({ entry: entry.entry }))],
				entry.name,
				{
					type: entry.fileType,
					lastModified: entry.lastModified,
				},
			);
			await storageService.saveMediaAsset({
				projectId,
				mediaAsset: {
					id: assetId,
					name: entry.name,
					type: entry.mediaType,
					file: mediaFile,
					...(entry.width !== undefined && { width: entry.width }),
					...(entry.height !== undefined && { height: entry.height }),
					...(entry.duration !== undefined && { duration: entry.duration }),
					...(entry.ephemeral !== undefined && { ephemeral: entry.ephemeral }),
					...(entry.thumbnailUrl !== undefined && {
						thumbnailUrl: entry.thumbnailUrl,
					}),
				},
			});
		}

		await storageService.saveProject({ project });
	} catch (error) {
		// Roll back anything already written under the fresh project id.
		try {
			await storageService.deleteProjectMedia({ projectId });
			await storageService.deleteProject({ id: projectId });
		} catch {
			// Cleanup is best-effort; surface the original failure.
		}
		throw error;
	}

	return { project, importedMediaCount: parsed.media.length };
}

function assertSerializedProject(
	record: unknown,
): asserts record is SerializedProject {
	if (!isRecord(record) || Array.isArray(record)) {
		throw new ProjectArchiveError({
			message: "Invalid project data (not a project object)",
		});
	}
	const { metadata, scenes, currentSceneId, settings, version } = record;
	if (
		!isRecord(metadata) ||
		Array.isArray(metadata) ||
		typeof metadata.name !== "string" ||
		metadata.name.length === 0
	) {
		throw new ProjectArchiveError({
			message: "Invalid project data (missing project name)",
		});
	}
	if (
		typeof metadata.createdAt !== "string" ||
		typeof metadata.updatedAt !== "string"
	) {
		throw new ProjectArchiveError({
			message: "Invalid project data (missing timestamps)",
		});
	}
	if (!Array.isArray(scenes)) {
		throw new ProjectArchiveError({
			message: "Invalid project data (missing scenes)",
		});
	}
	if (typeof currentSceneId !== "string") {
		throw new ProjectArchiveError({
			message: "Invalid project data (missing current scene)",
		});
	}
	if (!isRecord(settings) || Array.isArray(settings)) {
		throw new ProjectArchiveError({
			message: "Invalid project data (missing settings)",
		});
	}
	if (typeof version !== "number") {
		throw new ProjectArchiveError({
			message: "Invalid project data (missing version)",
		});
	}
}
