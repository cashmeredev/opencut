import type { StorageMigration } from "@/services/storage/migrations/base";
import type { ProjectRecord } from "@/services/storage/migrations/transformers/types";
import { getProjectId } from "@/services/storage/migrations/transformers/utils";

/**
 * In-memory counterpart to `runStorageMigrations`: applies the same migration
 * registry to a single project record (e.g. parsed from an imported project
 * archive) without touching IndexedDB.
 */

export function getRecordVersion({
	project,
}: {
	project: ProjectRecord;
}): number {
	const versionValue = project.version;
	if (typeof versionValue === "number") {
		return versionValue;
	}

	const scenesValue = project.scenes;
	if (Array.isArray(scenesValue) && scenesValue.length > 0) {
		return 1;
	}

	return 0;
}

export interface MigratedProjectRecord {
	project: ProjectRecord;
	fromVersion: number;
	toVersion: number;
	complete: boolean;
}

export async function migrateProjectRecord({
	project,
	migrations,
}: {
	project: ProjectRecord;
	migrations: StorageMigration[];
}): Promise<MigratedProjectRecord> {
	const orderedMigrations = [...migrations].sort((a, b) => a.from - b.from);
	const fromVersion = getRecordVersion({ project });
	const targetVersion = orderedMigrations.at(-1)?.to ?? fromVersion;

	let currentVersion = fromVersion;
	let record = project;
	const projectId = getProjectId({ project: record }) ?? "imported-project";

	for (const migration of orderedMigrations) {
		if (migration.from !== currentVersion) {
			continue;
		}

		const result = await migration.run({
			projectId,
			project: record,
		});

		if (result.skipped) {
			break;
		}

		record = result.project;
		currentVersion = migration.to;
	}

	return {
		project: record,
		fromVersion,
		toVersion: currentVersion,
		complete: currentVersion >= targetVersion,
	};
}
