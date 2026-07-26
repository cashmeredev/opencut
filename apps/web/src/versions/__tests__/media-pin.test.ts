import { beforeEach, describe, expect, test } from "bun:test";
import { EditorCore } from "@/core";
import { buildDefaultScene } from "@/timeline/scenes";
import type { ImageElement } from "@/timeline";
import type { MediaAsset } from "@/media/types";
import { mediaTimeFromSeconds, ZERO_MEDIA_TIME } from "@/wasm";
import { CURRENT_PROJECT_VERSION } from "@/services/storage/migrations";
import { InMemoryVersionStore } from "@/versions";

const PROJECT_ID = "project-1";
const IMAGE_DURATION = mediaTimeFromSeconds({ seconds: 5 });

function buildImageElement({ mediaId }: { mediaId: string }): ImageElement {
	return {
		id: `image-${mediaId}`,
		type: "image",
		name: `image-${mediaId}`,
		startTime: ZERO_MEDIA_TIME,
		duration: IMAGE_DURATION,
		trimStart: ZERO_MEDIA_TIME,
		trimEnd: ZERO_MEDIA_TIME,
		mediaId,
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

function buildImageAsset({ id }: { id: string }): MediaAsset {
	return {
		id,
		name: id,
		type: "image",
		file: new File([new Uint8Array([1, 2, 3])], `${id}.png`, {
			type: "image/png",
		}),
	};
}

function seedEditor(): { editor: EditorCore } {
	const editor = EditorCore.getInstance();
	editor.versions.setStore({ store: new InMemoryVersionStore() });
	editor.versions.setNow({ now: () => 1_000_000_000 });

	const scene = buildDefaultScene({ name: "Main scene", isMain: true });
	// Only media-pinned is referenced by the document (and thus snapshots);
	// media-free exists as an asset but is used nowhere.
	scene.tracks.main.elements = [buildImageElement({ mediaId: "media-pinned" })];
	editor.project.setActiveProject({
		project: {
			metadata: {
				id: PROJECT_ID,
				name: "Test project",
				duration: IMAGE_DURATION,
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
	editor.media.setAssets({
		assets: [
			buildImageAsset({ id: "media-pinned" }),
			buildImageAsset({ id: "media-free" }),
		],
	});
	return { editor };
}

describe("Media pinning", () => {
	beforeEach(() => {
		EditorCore.reset();
	});

	test("deleting a media asset referenced by a stored version is blocked", async () => {
		const { editor } = seedEditor();

		const version = await editor.versions.createNamedCheckpoint({
			name: "v1",
		});
		expect(version).not.toBeNull();

		editor.media.removeMediaAsset({ projectId: PROJECT_ID, id: "media-pinned" });
		expect(
			editor.media.getAssets().some((asset) => asset.id === "media-pinned"),
		).toBe(true);

		// An unreferenced asset in the same project deletes normally.
		editor.media.removeMediaAsset({ projectId: PROJECT_ID, id: "media-free" });
		expect(
			editor.media.getAssets().some((asset) => asset.id === "media-free"),
		).toBe(false);
	});

	test("an asset is unpinned once the referencing version is deleted", async () => {
		const { editor } = seedEditor();

		const version = await editor.versions.createNamedCheckpoint({
			name: "v1",
		});
		expect(version).not.toBeNull();
		if (!version) return;

		expect(
			editor.versions.isMediaIdPinned({
				projectId: PROJECT_ID,
				mediaId: "media-pinned",
			}),
		).toBe(true);

		await editor.versions.deleteVersion({ versionId: version.id });
		expect(
			editor.versions.isMediaIdPinned({
				projectId: PROJECT_ID,
				mediaId: "media-pinned",
			}),
		).toBe(false);

		editor.media.removeMediaAsset({ projectId: PROJECT_ID, id: "media-pinned" });
		expect(
			editor.media.getAssets().some((asset) => asset.id === "media-pinned"),
		).toBe(false);
	});
});
