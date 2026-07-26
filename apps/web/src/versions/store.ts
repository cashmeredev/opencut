import { IndexedDBAdapter } from "@/services/storage/indexeddb-adapter";
import type { ProjectVersion } from "./types";

/**
 * Persistence backend for project versions. The IndexedDB implementation is
 * the production store (its own database, created lazily — the v31→v32
 * migration stamps project records); the in-memory implementation backs
 * unit tests.
 */
export interface VersionStore {
	put({ version }: { version: ProjectVersion }): Promise<void>;
	getAllForProject({
		projectId,
	}: {
		projectId: string;
	}): Promise<ProjectVersion[]>;
	delete({ id }: { id: string }): Promise<void>;
	deleteForProject({ projectId }: { projectId: string }): Promise<void>;
}

export class IndexedDBVersionStore implements VersionStore {
	private adapter = new IndexedDBAdapter<ProjectVersion>({
		dbName: "video-editor-project-versions",
		storeName: "project-versions",
		version: 1,
	});

	async put({ version }: { version: ProjectVersion }): Promise<void> {
		await this.adapter.set({ key: version.id, value: version });
	}

	async getAllForProject({
		projectId,
	}: {
		projectId: string;
	}): Promise<ProjectVersion[]> {
		const all = await this.adapter.getAll();
		return all.filter(
			(version) =>
				typeof version === "object" &&
				version !== null &&
				version.projectId === projectId,
		);
	}

	async delete({ id }: { id: string }): Promise<void> {
		await this.adapter.remove(id);
	}

	async deleteForProject({ projectId }: { projectId: string }): Promise<void> {
		const versions = await this.getAllForProject({ projectId });
		await Promise.all(
			versions.map((version) => this.adapter.remove(version.id)),
		);
	}
}

export class InMemoryVersionStore implements VersionStore {
	private versions = new Map<string, ProjectVersion>();

	async put({ version }: { version: ProjectVersion }): Promise<void> {
		this.versions.set(version.id, version);
	}

	async getAllForProject({
		projectId,
	}: {
		projectId: string;
	}): Promise<ProjectVersion[]> {
		return [...this.versions.values()].filter(
			(version) => version.projectId === projectId,
		);
	}

	async delete({ id }: { id: string }): Promise<void> {
		this.versions.delete(id);
	}

	async deleteForProject({ projectId }: { projectId: string }): Promise<void> {
		for (const version of this.versions.values()) {
			if (version.projectId === projectId) {
				this.versions.delete(version.id);
			}
		}
	}
}
