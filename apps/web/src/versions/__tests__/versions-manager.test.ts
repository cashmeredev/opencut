import { beforeEach, describe, expect, test } from "bun:test";
import { EditorCore } from "@/core";
import { buildDefaultScene } from "@/timeline/scenes";
import type { VideoElement } from "@/timeline";
import { mediaTimeFromSeconds, ZERO_MEDIA_TIME } from "@/wasm";
import { serializeProject } from "@/services/storage/service";
import { CURRENT_PROJECT_VERSION } from "@/services/storage/migrations";
import {
	AUTO_CHECKPOINT_MIN_INTERVAL_MS,
	AUTO_CHECKPOINT_RETENTION,
	InMemoryVersionStore,
} from "@/versions";

const PROJECT_ID = "project-1";
const VIDEO_DURATION = mediaTimeFromSeconds({ seconds: 10 });
const SPLIT_TIME = mediaTimeFromSeconds({ seconds: 4 });

let nowMs = 1_000_000_000;

function buildVideoElement(): VideoElement {
	return {
		id: "video-1",
		type: "video",
		name: "video-1",
		startTime: ZERO_MEDIA_TIME,
		duration: VIDEO_DURATION,
		trimStart: ZERO_MEDIA_TIME,
		trimEnd: ZERO_MEDIA_TIME,
		mediaId: "media-video-1",
		params: {
			"transform.positionX": 0,
			"transform.positionY": 0,
			"transform.scaleX": 1,
			"transform.scaleY": 1,
			"transform.rotate": 0,
			opacity: 1,
		},
	};
}

function seedEditor(): { editor: EditorCore; store: InMemoryVersionStore } {
	const editor = EditorCore.getInstance();
	const store = new InMemoryVersionStore();
	editor.versions.setStore({ store });
	editor.versions.setNow({ now: () => nowMs });

	const scene = buildDefaultScene({ name: "Main scene", isMain: true });
	scene.tracks.main.elements = [buildVideoElement()];
	editor.project.setActiveProject({
		project: {
			metadata: {
				id: PROJECT_ID,
				name: "Test project",
				duration: VIDEO_DURATION,
				createdAt: new Date("2026-01-01T00:00:00Z"),
				updatedAt: new Date("2026-01-01T00:00:00Z"),
			},
			scenes: [scene],
			currentSceneId: scene.id,
			settings: {
				fps: { numerator: 30, denominator: 1 },
				canvasSize: { width: 1920, height: 1080 },
				background: { type: "color", color: "#000000" },
			},
			version: CURRENT_PROJECT_VERSION,
		},
	});
	editor.scenes.setScenes({ scenes: [scene], activeSceneId: scene.id });
	return { editor, store };
}

function splitSeededVideo({ editor }: { editor: EditorCore }): void {
	const scene = editor.scenes.getActiveScene();
	editor.timeline.splitElements({
		elements: [{ trackId: scene.tracks.main.id, elementId: "video-1" }],
		splitTime: SPLIT_TIME,
	});
}

describe("VersionsManager", () => {
	beforeEach(() => {
		EditorCore.reset();
	});

	test("restore round-trips the document through a named checkpoint", async () => {
		const { editor } = seedEditor();

		const version = await editor.versions.createNamedCheckpoint({
			name: "v1",
		});
		expect(version).not.toBeNull();
		if (!version) return;

		splitSeededVideo({ editor });
		expect(
			editor.scenes.getActiveScene().tracks.main.elements,
		).toHaveLength(2);

		const didRestore = await editor.versions.restoreVersion({
			versionId: version.id,
		});
		expect(didRestore).toBe(true);

		const restored = editor.project.getActive();
		expect(serializeProject({ project: restored })).toEqual(version.project);
	});

	test("restore writes a safety auto-checkpoint of the abandoned head", async () => {
		const { editor, store } = seedEditor();

		const version = await editor.versions.createNamedCheckpoint({
			name: "v1",
		});
		expect(version).not.toBeNull();
		if (!version) return;

		nowMs += AUTO_CHECKPOINT_MIN_INTERVAL_MS;
		splitSeededVideo({ editor });

		const didRestore = await editor.versions.restoreVersion({
			versionId: version.id,
		});
		expect(didRestore).toBe(true);

		const stored = await store.getAllForProject({ projectId: PROJECT_ID });
		const autos = stored.filter((candidate) => candidate.kind === "auto");
		expect(autos).toHaveLength(1);

		// The safety checkpoint captures the mutated (split) document…
		const safetyScene = autos[0].project.scenes[0];
		expect(safetyScene.tracks.main.elements).toHaveLength(2);
		// …while the live document is back at the checkpointed state.
		expect(
			editor.scenes.getActiveScene().tracks.main.elements,
		).toHaveLength(1);
	});

	test("auto-checkpoints rotate as a ring buffer and never evict named versions", async () => {
		const { editor, store } = seedEditor();

		nowMs += AUTO_CHECKPOINT_MIN_INTERVAL_MS;
		expect(await editor.versions.writeAutoCheckpoint()).toBe(true);
		const [firstAuto] = await store.getAllForProject({
			projectId: PROJECT_ID,
		});

		for (let i = 0; i < AUTO_CHECKPOINT_RETENTION; i++) {
			nowMs += AUTO_CHECKPOINT_MIN_INTERVAL_MS;
			expect(await editor.versions.writeAutoCheckpoint()).toBe(true);
		}

		let stored = await store.getAllForProject({ projectId: PROJECT_ID });
		let autos = stored.filter((candidate) => candidate.kind === "auto");
		expect(autos).toHaveLength(AUTO_CHECKPOINT_RETENTION);
		// The 21st write evicted the oldest auto-checkpoint.
		expect(autos.some((candidate) => candidate.id === firstAuto.id)).toBe(
			false,
		);

		const named = await editor.versions.createNamedCheckpoint({
			name: "keep-me",
		});
		expect(named).not.toBeNull();

		nowMs += AUTO_CHECKPOINT_MIN_INTERVAL_MS;
		expect(await editor.versions.writeAutoCheckpoint()).toBe(true);

		stored = await store.getAllForProject({ projectId: PROJECT_ID });
		autos = stored.filter((candidate) => candidate.kind === "auto");
		expect(autos).toHaveLength(AUTO_CHECKPOINT_RETENTION);
		expect(
			stored.some(
				(candidate) => candidate.kind === "named" && candidate.name === "keep-me",
			),
		).toBe(true);
	});

	test("auto-checkpoints are throttled to one per interval", async () => {
		const { editor, store } = seedEditor();

		expect(await editor.versions.writeAutoCheckpoint()).toBe(true);
		expect(await editor.versions.writeAutoCheckpoint()).toBe(false);

		nowMs += AUTO_CHECKPOINT_MIN_INTERVAL_MS - 1;
		expect(await editor.versions.writeAutoCheckpoint()).toBe(false);

		nowMs += 1;
		expect(await editor.versions.writeAutoCheckpoint()).toBe(true);

		const stored = await store.getAllForProject({ projectId: PROJECT_ID });
		const autos = stored.filter((candidate) => candidate.kind === "auto");
		expect(autos).toHaveLength(2);
	});

	test("auto-checkpoints cannot be renamed or deleted", async () => {
		const { editor } = seedEditor();

		expect(await editor.versions.writeAutoCheckpoint()).toBe(true);
		const [auto] = editor.versions.getVersions({ projectId: PROJECT_ID });
		expect(auto.kind).toBe("auto");

		expect(
			await editor.versions.renameVersion({
				versionId: auto.id,
				name: "nope",
			}),
		).toBe(false);
		expect(await editor.versions.deleteVersion({ versionId: auto.id })).toBe(
			false,
		);
		expect(
			editor.versions.getVersions({ projectId: PROJECT_ID }),
		).toHaveLength(1);
	});

	test("named checkpoints can be renamed and deleted", async () => {
		const { editor } = seedEditor();

		const version = await editor.versions.createNamedCheckpoint({
			name: "v1",
		});
		expect(version).not.toBeNull();
		if (!version) return;

		expect(
			await editor.versions.renameVersion({
				versionId: version.id,
				name: "v1-final",
			}),
		).toBe(true);
		expect(
			editor.versions.getVersions({ projectId: PROJECT_ID })[0].name,
		).toBe("v1-final");

		expect(
			await editor.versions.deleteVersion({ versionId: version.id }),
		).toBe(true);
		expect(
			editor.versions.getVersions({ projectId: PROJECT_ID }),
		).toHaveLength(0);
	});
});
