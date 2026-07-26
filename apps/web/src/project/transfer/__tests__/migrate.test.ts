import { describe, expect, test } from "bun:test";
import {
	StorageMigration,
	type StorageMigrationRunArgs,
} from "@/services/storage/migrations/base";
import type {
	MigrationResult,
	ProjectRecord,
} from "@/services/storage/migrations/transformers/types";
import {
	CURRENT_PROJECT_VERSION,
	migrations,
} from "@/services/storage/migrations";
import { v5Project } from "@/services/storage/migrations/__tests__/fixtures";
import { getRecordVersion, migrateProjectRecord } from "../migrate";

class StepMigration extends StorageMigration {
	from: number;
	to: number;
	private transform: ({ project }: { project: ProjectRecord }) => ProjectRecord;
	private shouldSkip: boolean;

	constructor({
		from,
		to,
		transform,
		shouldSkip = false,
	}: {
		from: number;
		to: number;
		transform: ({ project }: { project: ProjectRecord }) => ProjectRecord;
		shouldSkip?: boolean;
	}) {
		super();
		this.from = from;
		this.to = to;
		this.transform = transform;
		this.shouldSkip = shouldSkip;
	}

	async run({
		project,
	}: StorageMigrationRunArgs): Promise<MigrationResult<ProjectRecord>> {
		if (this.shouldSkip) {
			return { project, skipped: true, reason: "skip requested" };
		}
		return { project: this.transform({ project }), skipped: false };
	}
}

function setVersion({
	project,
	version,
}: {
	project: ProjectRecord;
	version: number;
}): ProjectRecord {
	return { ...project, version };
}

describe("getRecordVersion", () => {
	test("uses the explicit version field", () => {
		expect(getRecordVersion({ project: { version: 7 } })).toBe(7);
	});

	test("detects v1 projects by their scenes array", () => {
		expect(getRecordVersion({ project: { scenes: [{ id: "s" }] } })).toBe(1);
	});

	test("falls back to v0", () => {
		expect(getRecordVersion({ project: {} })).toBe(0);
		expect(getRecordVersion({ project: { scenes: [] } })).toBe(0);
	});
});

describe("migrateProjectRecord", () => {
	test("applies migrations in version order regardless of registration order", async () => {
		const registry = [
			new StepMigration({
				from: 1,
				to: 2,
				transform: ({ project }) => setVersion({ project, version: 2 }),
			}),
			new StepMigration({
				from: 0,
				to: 1,
				transform: ({ project }) => setVersion({ project, version: 1 }),
			}),
		];

		const result = await migrateProjectRecord({
			project: { version: 0, steps: [] },
			migrations: registry,
		});

		expect(result.fromVersion).toBe(0);
		expect(result.toVersion).toBe(2);
		expect(result.complete).toBe(true);
		expect(result.project.version).toBe(2);
	});

	test("reports incomplete when a migration skips", async () => {
		const registry = [
			new StepMigration({
				from: 0,
				to: 1,
				transform: ({ project }) => project,
				shouldSkip: true,
			}),
			new StepMigration({
				from: 1,
				to: 2,
				transform: ({ project }) => setVersion({ project, version: 2 }),
			}),
		];

		const result = await migrateProjectRecord({
			project: { version: 0 },
			migrations: registry,
		});

		expect(result.toVersion).toBe(0);
		expect(result.complete).toBe(false);
	});

	test("is a no-op for current-version documents", async () => {
		const registry = [
			new StepMigration({
				from: 0,
				to: 1,
				transform: ({ project }) => setVersion({ project, version: 1 }),
			}),
		];

		const project = { version: 1, marker: "untouched" };
		const result = await migrateProjectRecord({ project, migrations: registry });

		expect(result.complete).toBe(true);
		expect(result.project).toBe(project);
	});

	test("migrates a real v5 fixture to the current version with the app registry", async () => {
		const result = await migrateProjectRecord({
			project: structuredClone(v5Project),
			migrations,
		});

		expect(result.fromVersion).toBe(5);
		expect(result.complete).toBe(true);
		expect(result.toVersion).toBe(CURRENT_PROJECT_VERSION);
		expect(result.project.version).toBe(CURRENT_PROJECT_VERSION);
	});
});
