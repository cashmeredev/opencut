import { beforeEach, describe, expect, test } from "bun:test";
import { EditorCore } from "@/core";
import {
	AddMediaAssetCommand,
	BatchCommand,
	InsertElementCommand,
} from "@/commands";
import { buildDefaultScene } from "@/timeline/scenes";
import { buildElementFromMedia } from "@/timeline";
import { mediaTimeFromSeconds, ZERO_MEDIA_TIME, type MediaTime } from "@/wasm";
import { storageService } from "@/services/storage/service";

const RECORDING_DURATION_SECONDS = 12.5;
const RECORDING_DURATION = mediaTimeFromSeconds({
	seconds: RECORDING_DURATION_SECONDS,
});
const PLAYHEAD_TIME = mediaTimeFromSeconds({ seconds: 2 });

function seedEditorWithEmptyScene(): EditorCore {
	const editor = EditorCore.getInstance();
	const scene = buildDefaultScene({ name: "Main scene", isMain: true });
	editor.project.setActiveProject({
		project: {
			metadata: {
				id: "project-1",
				name: "Test project",
				duration: ZERO_MEDIA_TIME,
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
			version: 1,
		},
	});
	editor.scenes.setScenes({ scenes: [scene], activeSceneId: scene.id });
	editor.media.setAssets({ assets: [] });
	return editor;
}

/**
 * Mirrors the stop-recording branch of the "voice-over" action handler in
 * use-editor-actions.ts: one undoable batch that adds the recorded audio as a
 * media asset and inserts it at the playhead with automatic audio placement.
 */
function buildVoiceOverBatch({
	projectId,
	startTime,
}: {
	projectId: string;
	startTime: MediaTime;
}): { batch: BatchCommand; assetId: string } {
	const file = new File([new Blob(["fake-audio"])], "voice-over.webm", {
		type: "audio/webm",
	});
	const addMediaCommand = new AddMediaAssetCommand({
		projectId,
		asset: {
			file,
			name: "Voice over",
			type: "audio",
			url: "blob:voice-over-test",
			duration: RECORDING_DURATION_SECONDS,
		},
	});
	const insertCommand = new InsertElementCommand({
		element: buildElementFromMedia({
			mediaId: addMediaCommand.getAssetId(),
			mediaType: "audio",
			name: "Voice over",
			duration: RECORDING_DURATION,
			startTime,
		}),
		placement: { mode: "auto", trackType: "audio" },
	});
	return {
		batch: new BatchCommand([addMediaCommand, insertCommand]),
		assetId: addMediaCommand.getAssetId(),
	};
}

describe("voice-over batch command", () => {
	beforeEach(() => {
		EditorCore.reset();
		// IndexedDB is unavailable under bun:test; the command treats a failed
		// save as fatal and rolls back the asset, so stub persistence out.
		storageService.saveMediaAsset = () => Promise.resolve();
		storageService.deleteMediaAsset = () => Promise.resolve();
	});

	test("adds an audio asset and inserts it on a new audio track at the playhead", () => {
		const editor = seedEditorWithEmptyScene();
		const { batch, assetId } = buildVoiceOverBatch({
			projectId: "project-1",
			startTime: PLAYHEAD_TIME,
		});

		batch.execute();

		const assets = editor.media.getAssets();
		expect(assets).toHaveLength(1);
		expect(assets[0].id).toBe(assetId);
		expect(assets[0].type).toBe("audio");
		expect(assets[0].name).toBe("Voice over");
		expect(assets[0].duration).toBe(RECORDING_DURATION_SECONDS);

		const { tracks } = editor.scenes.getActiveScene();
		expect(tracks.audio).toHaveLength(1);
		expect(tracks.audio[0].elements).toHaveLength(1);
		const element = tracks.audio[0].elements[0];
		expect(element.type).toBe("audio");
		expect(element.startTime).toBe(PLAYHEAD_TIME);
		expect(element.duration).toBe(RECORDING_DURATION);
		if (element.type === "audio" && element.sourceType === "upload") {
			expect(element.mediaId).toBe(assetId);
		} else {
			throw new Error("expected an upload audio element");
		}
	});

	test("undo removes both the element and the asset as a single step", () => {
		const editor = seedEditorWithEmptyScene();
		const { batch } = buildVoiceOverBatch({
			projectId: "project-1",
			startTime: PLAYHEAD_TIME,
		});

		batch.execute();
		batch.undo();

		expect(editor.media.getAssets()).toHaveLength(0);
		const { tracks } = editor.scenes.getActiveScene();
		expect(
			tracks.audio.flatMap((track) => track.elements),
		).toHaveLength(0);
	});

	test("redo re-inserts the element and restores the asset", () => {
		const editor = seedEditorWithEmptyScene();
		const { batch, assetId } = buildVoiceOverBatch({
			projectId: "project-1",
			startTime: PLAYHEAD_TIME,
		});

		batch.execute();
		batch.undo();
		batch.redo();

		expect(editor.media.getAssets().map((asset) => asset.id)).toEqual([
			assetId,
		]);
		const { tracks } = editor.scenes.getActiveScene();
		const elements = tracks.audio.flatMap((track) => track.elements);
		expect(elements).toHaveLength(1);
		expect(elements[0].startTime).toBe(PLAYHEAD_TIME);
	});

	test("reuses an existing audio track when the slot is free", () => {
		const editor = seedEditorWithEmptyScene();
		const first = buildVoiceOverBatch({
			projectId: "project-1",
			startTime: PLAYHEAD_TIME,
		});
		first.batch.execute();

		const secondStart = mediaTimeFromSeconds({
			seconds: 2 + RECORDING_DURATION_SECONDS + 1,
		});
		const second = buildVoiceOverBatch({
			projectId: "project-1",
			startTime: secondStart,
		});
		second.batch.execute();

		const { tracks } = editor.scenes.getActiveScene();
		expect(tracks.audio).toHaveLength(1);
		expect(tracks.audio[0].elements).toHaveLength(2);
		expect(
			tracks.audio[0].elements.map((element) => element.startTime),
		).toEqual([PLAYHEAD_TIME, secondStart]);
	});
});
