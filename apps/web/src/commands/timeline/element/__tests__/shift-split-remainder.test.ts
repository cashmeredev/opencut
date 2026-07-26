import { beforeEach, describe, expect, test } from "bun:test";
import { EditorCore } from "@/core";
import {
	BatchCommand,
	InsertElementCommand,
	ShiftSplitRemainderCommand,
	SplitElementsCommand,
} from "@/commands";
import { buildDefaultScene } from "@/timeline/scenes";
import { buildElementFromMedia, type VideoElement } from "@/timeline";
import {
	addMediaTime,
	mediaTimeFromSeconds,
	ZERO_MEDIA_TIME,
} from "@/wasm";

const SPLIT_TIME = mediaTimeFromSeconds({ seconds: 4 });
const FREEZE_DURATION = mediaTimeFromSeconds({ seconds: 3 });
const VIDEO_DURATION = mediaTimeFromSeconds({ seconds: 10 });

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

function seedSceneWithVideo(): { editor: EditorCore; trackId: string } {
	const editor = EditorCore.getInstance();
	const scene = buildDefaultScene({ name: "Main scene", isMain: true });
	scene.tracks.main.elements = [buildVideoElement()];
	editor.project.setActiveProject({
		project: {
			metadata: {
				id: "project-1",
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
			version: 1,
		},
	});
	editor.scenes.setScenes({ scenes: [scene], activeSceneId: scene.id });
	return { editor, trackId: scene.tracks.main.id };
}

function buildFreezeBatch({ trackId }: { trackId: string }): BatchCommand {
	const splitCommand = new SplitElementsCommand({
		elements: [{ trackId, elementId: "video-1" }],
		splitTime: SPLIT_TIME,
	});
	const shiftCommand = new ShiftSplitRemainderCommand({
		split: splitCommand,
		newStartTime: addMediaTime({ a: SPLIT_TIME, b: FREEZE_DURATION }),
	});
	const insertCommand = new InsertElementCommand({
		element: buildElementFromMedia({
			mediaId: "media-freeze-1",
			mediaType: "image",
			name: "Freeze frame",
			duration: FREEZE_DURATION,
			startTime: SPLIT_TIME,
		}),
		placement: { mode: "explicit", trackId },
	});
	return new BatchCommand([splitCommand, shiftCommand, insertCommand]);
}

describe("ShiftSplitRemainderCommand", () => {
	beforeEach(() => {
		EditorCore.reset();
	});

	test("splits, shifts the right side, and inserts into the gap", () => {
		const { editor, trackId } = seedSceneWithVideo();

		buildFreezeBatch({ trackId }).execute();

		const elements = editor.scenes
			.getActiveScene()
			.tracks.main.elements.slice()
			.sort((a, b) => (a.startTime > b.startTime ? 1 : -1));
		expect(elements.map((element) => element.type)).toEqual([
			"video",
			"image",
			"video",
		]);

		const [left, image, right] = elements;
		expect(left.startTime).toBe(ZERO_MEDIA_TIME);
		expect(left.duration).toBe(SPLIT_TIME);
		expect(image.startTime).toBe(SPLIT_TIME);
		expect(image.duration).toBe(FREEZE_DURATION);
		expect(right.startTime).toBe(
			addMediaTime({ a: SPLIT_TIME, b: FREEZE_DURATION }),
		);
		expect(right.duration).toBe(
			addMediaTime({ a: VIDEO_DURATION, b: mediaTimeFromSeconds({ seconds: -4 }) }),
		);
	});

	test("undo restores the original element as a single step", () => {
		const { editor, trackId } = seedSceneWithVideo();
		const batch = buildFreezeBatch({ trackId });

		batch.execute();
		batch.undo();

		const elements = editor.scenes.getActiveScene().tracks.main.elements;
		expect(elements).toHaveLength(1);
		expect(elements[0].id).toBe("video-1");
		expect(elements[0].startTime).toBe(ZERO_MEDIA_TIME);
		expect(elements[0].duration).toBe(VIDEO_DURATION);
	});

	test("redo after undo re-applies the shift to the re-split elements", () => {
		const { editor, trackId } = seedSceneWithVideo();
		const batch = buildFreezeBatch({ trackId });

		batch.execute();
		batch.undo();
		batch.redo();

		const elements = editor.scenes
			.getActiveScene()
			.tracks.main.elements.slice()
			.sort((a, b) => (a.startTime > b.startTime ? 1 : -1));
		expect(elements.map((element) => element.type)).toEqual([
			"video",
			"image",
			"video",
		]);
		// The re-executed split generates fresh right-side ids; the shift must
		// still land on them.
		const right = elements.at(-1);
		expect(right?.type).toBe("video");
		expect(right?.startTime).toBe(
			addMediaTime({ a: SPLIT_TIME, b: FREEZE_DURATION }),
		);
	});

	test("is a no-op when the split produces no right side", () => {
		const { editor, trackId } = seedSceneWithVideo();

		const splitCommand = new SplitElementsCommand({
			elements: [{ trackId, elementId: "video-1" }],
			splitTime: ZERO_MEDIA_TIME,
		});
		const shiftCommand = new ShiftSplitRemainderCommand({
			split: splitCommand,
			newStartTime: FREEZE_DURATION,
		});
		const batch = new BatchCommand([splitCommand, shiftCommand]);

		expect(() => {
			batch.execute();
			batch.undo();
		}).not.toThrow();
		expect(
			editor.scenes.getActiveScene().tracks.main.elements,
		).toHaveLength(1);
	});
});
