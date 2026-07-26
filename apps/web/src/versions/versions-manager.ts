import type { EditorCore } from "@/core";
import { toast } from "sonner";
import { generateUUID } from "@/utils/id";
import type { TProject } from "@/project/types";
import {
	deserializeProject,
	serializeProject,
} from "@/services/storage/service";
import { isStorageQuotaExceededError } from "@/services/storage/quota";
import { getProjectDurationFromScenes } from "@/timeline/scenes";
import { ZERO_MEDIA_TIME } from "@/wasm";
import { getReferencedMediaIds } from "./media-references";
import {
	IndexedDBVersionStore,
	InMemoryVersionStore,
	type VersionStore,
} from "./store";
import type { ProjectVersion } from "./types";

export const AUTO_CHECKPOINT_RETENTION = 20;
export const AUTO_CHECKPOINT_MIN_INTERVAL_MS = 60_000;

const VERSION_THUMBNAIL_MAX_WIDTH = 320;

async function captureVersionThumbnail({
	editor,
}: {
	editor: EditorCore;
}): Promise<string | undefined> {
	try {
		const blob = await editor.renderer.captureFrame();
		if (!blob) return undefined;

		const bitmap = await createImageBitmap(blob);
		const scale = Math.min(1, VERSION_THUMBNAIL_MAX_WIDTH / bitmap.width);
		const canvas = document.createElement("canvas");
		canvas.width = Math.max(1, Math.round(bitmap.width * scale));
		canvas.height = Math.max(1, Math.round(bitmap.height * scale));
		const context = canvas.getContext("2d");
		if (!context) {
			bitmap.close();
			return undefined;
		}
		context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
		bitmap.close();
		return canvas.toDataURL("image/jpeg", 0.75);
	} catch (error) {
		console.error("Failed to capture version thumbnail:", error);
		return undefined;
	}
}

/**
 * Named checkpoints + throttled auto-checkpoints (ring buffer) per project.
 * Restore is a wholesale document swap that deliberately bypasses undo/redo:
 * undoing a restore = the safety auto-checkpoint written before the jump.
 * See notes/project-versioning.md.
 */
export class VersionsManager {
	private versionsByProject = new Map<string, ProjectVersion[]>();
	private pinnedMediaByProject = new Map<string, Set<string>>();
	private lastAutoCheckpointAt = new Map<string, number>();
	private hydratedProjectId: string | null = null;
	private listeners = new Set<() => void>();
	private store: VersionStore;
	private now: () => number;

	constructor({
		editor,
		store,
		now,
	}: {
		editor: EditorCore;
		store?: VersionStore;
		now?: () => number;
	}) {
		this.editor = editor;
		// Without IndexedDB (unit tests, unsupported browsers) persistence is
		// impossible; fall back to the in-memory store instead of failing.
		this.store =
			store ??
			(typeof indexedDB === "undefined"
				? new InMemoryVersionStore()
				: new IndexedDBVersionStore());
		this.now = now ?? (() => Date.now());

		this.editor.project.subscribe(() => {
			const activeProjectId =
				this.editor.project.getActiveOrNull()?.metadata.id ?? null;
			if (activeProjectId === this.hydratedProjectId) return;
			this.hydratedProjectId = activeProjectId;
			if (activeProjectId) {
				this.listVersions({ projectId: activeProjectId }).catch((error) => {
					console.error("Failed to load project versions:", error);
				});
			}
		});
	}

	private editor: EditorCore;

	/** Test seam: swap the persistence backend / clock. */
	setStore({ store }: { store: VersionStore }): void {
		this.store = store;
	}

	setNow({ now }: { now: () => number }): void {
		this.now = now;
	}

	subscribe(listener: () => void): () => void {
		this.listeners.add(listener);
		return () => this.listeners.delete(listener);
	}

	private notify(): void {
		this.listeners.forEach((listener) => {
			listener();
		});
	}

	/** Cached versions for a project, newest first. */
	getVersions({ projectId }: { projectId: string }): ProjectVersion[] {
		return this.versionsByProject.get(projectId) ?? [];
	}

	/** Reloads versions for a project from the store into the cache. */
	async listVersions({
		projectId,
	}: {
		projectId: string;
	}): Promise<ProjectVersion[]> {
		try {
			const versions = sortNewestFirst({
				versions: await this.store.getAllForProject({ projectId }),
			});
			this.setProjectVersions({ projectId, versions });
			return versions;
		} catch (error) {
			console.error("Failed to list project versions:", error);
			return this.getVersions({ projectId });
		}
	}

	async createNamedCheckpoint({
		name,
	}: {
		name: string;
	}): Promise<ProjectVersion | null> {
		const trimmedName = name.trim();
		if (!trimmedName) {
			toast.error("Checkpoint needs a name");
			return null;
		}

		const thumbnail = await captureVersionThumbnail({ editor: this.editor });
		const version = this.buildSnapshot({
			kind: "named",
			name: trimmedName,
			thumbnail,
		});
		if (!version) return null;

		const didWrite = await this.putVersion({ version });
		if (!didWrite) return null;

		toast.success(`Checkpoint "${trimmedName}" created`);
		return version;
	}

	/**
	 * Writes an auto-checkpoint of the current head, throttled to at most one
	 * per AUTO_CHECKPOINT_MIN_INTERVAL_MS and project. `force` skips the
	 * throttle (used for the safety checkpoint before a restore jump).
	 * Returns false when throttled, when there is nothing to snapshot, or when
	 * the write failed.
	 */
	async writeAutoCheckpoint({
		force = false,
	}: { force?: boolean } = {}): Promise<boolean> {
		const activeProject = this.editor.project.getActiveOrNull();
		if (!activeProject) return false;

		const projectId = activeProject.metadata.id;
		const nowMs = this.now();
		const lastAt = this.lastAutoCheckpointAt.get(projectId);
		if (
			!force &&
			lastAt !== undefined &&
			nowMs - lastAt < AUTO_CHECKPOINT_MIN_INTERVAL_MS
		) {
			return false;
		}

		const version = this.buildSnapshot({ kind: "auto" });
		if (!version) return false;

		const didWrite = await this.putVersion({ version });
		if (!didWrite) return false;

		this.lastAutoCheckpointAt.set(projectId, nowMs);
		await this.rotateAutoCheckpoints({ projectId });
		return true;
	}

	/**
	 * Jumps head to a stored version: writes a safety auto-checkpoint of the
	 * current head first, then swaps the document wholesale (bypassing
	 * undo/redo — undo of a restore = that safety checkpoint).
	 */
	async restoreVersion({ versionId }: { versionId: string }): Promise<boolean> {
		const activeProject = this.editor.project.getActiveOrNull();
		if (!activeProject) return false;

		const version = this.getVersions({
			projectId: activeProject.metadata.id,
		}).find((candidate) => candidate.id === versionId);
		if (!version) {
			toast.error("Version not found");
			return false;
		}

		const didWriteSafetyCheckpoint = await this.writeAutoCheckpoint({
			force: true,
		});
		if (!didWriteSafetyCheckpoint) {
			toast.error("Couldn't save a safety checkpoint of the current state", {
				description: "Restore aborted so no work is lost.",
			});
			return false;
		}

		const project = deserializeProject({ serialized: version.project });
		this.editor.project.setActiveProject({ project });
		this.editor.scenes.initializeScenes({
			scenes: project.scenes,
			currentSceneId: project.currentSceneId,
		});
		this.editor.playback.seek({ time: ZERO_MEDIA_TIME });
		this.editor.command.clear();
		this.editor.selection.clearSelection();
		this.editor.save.markDirty({ force: true });
		this.notify();

		toast.success(`Restored ${getVersionLabel({ version })}`, {
			description: "Previous state saved as an auto-checkpoint.",
		});
		return true;
	}

	async renameVersion({
		versionId,
		name,
	}: {
		versionId: string;
		name: string;
	}): Promise<boolean> {
		const trimmedName = name.trim();
		if (!trimmedName) {
			toast.error("Checkpoint needs a name");
			return false;
		}

		const found = this.findVersion({ versionId });
		if (!found) {
			toast.error("Version not found");
			return false;
		}
		if (found.version.kind !== "named") {
			toast.error("Auto-checkpoints can't be renamed");
			return false;
		}

		const renamed: ProjectVersion = { ...found.version, name: trimmedName };
		try {
			await this.store.put({ version: renamed });
		} catch (error) {
			this.toastStoreError({ error, fallback: "Failed to rename checkpoint" });
			return false;
		}

		this.replaceCachedVersion({ version: renamed });
		return true;
	}

	/** Deletes a named checkpoint. Auto-checkpoints only leave via rotation. */
	async deleteVersion({ versionId }: { versionId: string }): Promise<boolean> {
		const found = this.findVersion({ versionId });
		if (!found) {
			toast.error("Version not found");
			return false;
		}
		if (found.version.kind !== "named") {
			toast.error("Auto-checkpoints can't be deleted", {
				description: "They rotate out automatically.",
			});
			return false;
		}

		try {
			await this.store.delete({ id: versionId });
		} catch (error) {
			this.toastStoreError({ error, fallback: "Failed to delete checkpoint" });
			return false;
		}

		const remaining = this.getVersions({ projectId: found.projectId }).filter(
			(version) => version.id !== versionId,
		);
		this.setProjectVersions({ projectId: found.projectId, versions: remaining });
		return true;
	}

	async deleteVersionsForProject({
		projectId,
	}: {
		projectId: string;
	}): Promise<void> {
		try {
			await this.store.deleteForProject({ projectId });
		} catch (error) {
			console.error("Failed to delete project versions:", error);
		}
		this.versionsByProject.delete(projectId);
		this.pinnedMediaByProject.delete(projectId);
		this.lastAutoCheckpointAt.delete(projectId);
		this.notify();
	}

	/**
	 * Sync pin check backed by the cached version list (hydrated on project
	 * load). Deleting a pinned media asset is blocked so a restore can never
	 * hit a missing blob.
	 */
	isMediaIdPinned({
		projectId,
		mediaId,
	}: {
		projectId: string;
		mediaId: string;
	}): boolean {
		return this.pinnedMediaByProject.get(projectId)?.has(mediaId) ?? false;
	}

	private buildSnapshot({
		kind,
		name,
		thumbnail,
	}: {
		kind: ProjectVersion["kind"];
		name?: string;
		thumbnail?: string;
	}): ProjectVersion | null {
		const activeProject = this.editor.project.getActiveOrNull();
		if (!activeProject) return null;

		const scenes = this.editor.scenes.getScenes();
		const project: TProject = {
			...activeProject,
			scenes,
			metadata: {
				...activeProject.metadata,
				duration: getProjectDurationFromScenes({ scenes }),
				updatedAt: new Date(this.now()),
			},
		};

		return {
			id: generateUUID(),
			projectId: activeProject.metadata.id,
			kind,
			...(name !== undefined && { name }),
			createdAt: new Date(this.now()),
			project: serializeProject({ project }),
			...(thumbnail !== undefined && { thumbnail }),
		};
	}

	/** Persists a version and reflects it in the cache. Returns false on failure. */
	private async putVersion({
		version,
	}: {
		version: ProjectVersion;
	}): Promise<boolean> {
		try {
			await this.store.put({ version });
		} catch (error) {
			this.toastStoreError({ error, fallback: "Failed to save checkpoint" });
			return false;
		}

		const versions = [
			version,
			...this.getVersions({ projectId: version.projectId }),
		];
		this.setProjectVersions({
			projectId: version.projectId,
			versions: sortNewestFirst({ versions }),
		});
		return true;
	}

	/** Ring buffer: keep the newest AUTO_CHECKPOINT_RETENTION autos per project. */
	private async rotateAutoCheckpoints({
		projectId,
	}: {
		projectId: string;
	}): Promise<void> {
		const versions = this.getVersions({ projectId });
		const autoVersions = versions.filter((version) => version.kind === "auto");
		const evicted = autoVersions.slice(AUTO_CHECKPOINT_RETENTION);
		if (evicted.length === 0) return;

		await Promise.all(
			evicted.map(async (version) => {
				try {
					await this.store.delete({ id: version.id });
				} catch (error) {
					console.error("Failed to rotate auto-checkpoint:", error);
				}
			}),
		);

		const evictedIds = new Set(evicted.map((version) => version.id));
		this.setProjectVersions({
			projectId,
			versions: versions.filter((version) => !evictedIds.has(version.id)),
		});
	}

	private findVersion({
		versionId,
	}: {
		versionId: string;
	}): { projectId: string; version: ProjectVersion } | null {
		for (const [projectId, versions] of this.versionsByProject) {
			const version = versions.find((candidate) => candidate.id === versionId);
			if (version) return { projectId, version };
		}
		return null;
	}

	private replaceCachedVersion({ version }: { version: ProjectVersion }): void {
		const versions = this.getVersions({ projectId: version.projectId }).map(
			(candidate) => (candidate.id === version.id ? version : candidate),
		);
		this.setProjectVersions({ projectId: version.projectId, versions });
	}

	private setProjectVersions({
		projectId,
		versions,
	}: {
		projectId: string;
		versions: ProjectVersion[];
	}): void {
		this.versionsByProject.set(projectId, versions);
		const pinned = new Set<string>();
		for (const version of versions) {
			for (const mediaId of getReferencedMediaIds({
				project: version.project,
			})) {
				pinned.add(mediaId);
			}
		}
		this.pinnedMediaByProject.set(projectId, pinned);
		this.notify();
	}

	private toastStoreError({
		error,
		fallback,
	}: {
		error: unknown;
		fallback: string;
	}): void {
		if (isStorageQuotaExceededError({ error })) {
			toast.error("Not enough browser storage", {
				description: error instanceof Error ? error.message : undefined,
			});
			return;
		}
		console.error(`${fallback}:`, error);
		toast.error(fallback);
	}
}

function sortNewestFirst({
	versions,
}: {
	versions: ProjectVersion[];
}): ProjectVersion[] {
	return [...versions].sort(
		(a, b) =>
			new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
	);
}

function getVersionLabel({ version }: { version: ProjectVersion }): string {
	return version.kind === "named" && version.name
		? `"${version.name}"`
		: "auto-checkpoint";
}
