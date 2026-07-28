import { beforeEach, describe, expect, test } from "bun:test";
import type { FrameRate } from "opencut-wasm";
import { EditorCore } from "@/core";
import { UpdateElementsCommand } from "@/commands";
import { buildDefaultScene } from "@/timeline/scenes";
import type { ImageElement, TimelineElement, VideoElement } from "@/timeline";
import { buildResizeMembers } from "@/timeline/controllers/resize-controller";
import { computeGroupResize } from "@/timeline/group-resize";
import type {
	GroupResizeMember,
	GroupResizePushTarget,
	GroupResizeUpdate,
} from "@/timeline/group-resize";
import {
	addMediaTime,
	type MediaTime,
	mediaTime,
	mediaTimeFromSeconds,
	TICKS_PER_SECOND,
	ZERO_MEDIA_TIME,
} from "@/wasm";

const FPS: FrameRate = { numerator: 30, denominator: 1 };

function seconds({ value }: { value: number }): MediaTime {
	return mediaTimeFromSeconds({ seconds: value });
}

function buildPushTarget({
	elementId,
	startTime,
	duration,
}: {
	elementId: string;
	startTime: MediaTime;
	duration: MediaTime;
}): GroupResizePushTarget {
	return {
		trackId: "track-1",
		elementId,
		startTime,
		duration,
		trimStart: ZERO_MEDIA_TIME,
		trimEnd: ZERO_MEDIA_TIME,
	};
}

function buildStaticMember({
	startTime,
	duration,
	leftNeighborBound = null,
	rightNeighborBound,
	rightPushChain,
}: {
	startTime: MediaTime;
	duration: MediaTime;
	leftNeighborBound?: MediaTime | null;
	rightNeighborBound: MediaTime | null;
	rightPushChain?: GroupResizePushTarget[];
}): GroupResizeMember {
	return {
		trackId: "track-1",
		elementId: "element-1",
		startTime,
		duration,
		trimStart: ZERO_MEDIA_TIME,
		trimEnd: ZERO_MEDIA_TIME,
		leftNeighborBound,
		rightNeighborBound,
		rightPushChain,
	};
}

function buildVideoMember({
	startTime,
	duration,
	sourceDuration,
	rightNeighborBound,
}: {
	startTime: MediaTime;
	duration: MediaTime;
	sourceDuration: MediaTime;
	rightNeighborBound: MediaTime | null;
}): GroupResizeMember {
	return {
		trackId: "track-1",
		elementId: "element-1",
		startTime,
		duration,
		trimStart: ZERO_MEDIA_TIME,
		trimEnd: ZERO_MEDIA_TIME,
		sourceDuration,
		leftNeighborBound: null,
		rightNeighborBound,
	};
}

function findUpdate({
	updates,
	elementId,
}: {
	updates: GroupResizeUpdate[];
	elementId: string;
}): GroupResizeUpdate | undefined {
	return updates.find((update) => update.elementId === elementId);
}

describe("computeGroupResize ripple push", () => {
	test("extends into the gap before pushing anything", () => {
		const member = buildStaticMember({
			startTime: ZERO_MEDIA_TIME,
			duration: seconds({ value: 5 }),
			rightNeighborBound: seconds({ value: 10 }),
			rightPushChain: [
				buildPushTarget({
					elementId: "neighbor-1",
					startTime: seconds({ value: 10 }),
					duration: seconds({ value: 10 }),
				}),
			],
		});

		const result = computeGroupResize({
			members: [member],
			side: "right",
			deltaTime: seconds({ value: 3 }),
			fps: FPS,
		});

		expect(result.deltaTime).toBe(seconds({ value: 3 }));
		expect(result.updates).toHaveLength(1);
		expect(
			findUpdate({ updates: result.updates, elementId: "element-1" })?.patch
				.duration,
		).toBe(seconds({ value: 8 }));
	});

	test("pushes the flush right neighbor by the overflow", () => {
		const member = buildStaticMember({
			startTime: ZERO_MEDIA_TIME,
			duration: seconds({ value: 5 }),
			rightNeighborBound: seconds({ value: 5 }),
			rightPushChain: [
				buildPushTarget({
					elementId: "neighbor-1",
					startTime: seconds({ value: 5 }),
					duration: seconds({ value: 10 }),
				}),
			],
		});

		const result = computeGroupResize({
			members: [member],
			side: "right",
			deltaTime: seconds({ value: 3 }),
			fps: FPS,
		});

		expect(result.deltaTime).toBe(seconds({ value: 3 }));
		expect(result.updates).toHaveLength(2);
		expect(
			findUpdate({ updates: result.updates, elementId: "element-1" })?.patch
				.duration,
		).toBe(seconds({ value: 8 }));
		const push = findUpdate({ updates: result.updates, elementId: "neighbor-1" });
		expect(push?.patch.startTime).toBe(seconds({ value: 8 }));
		expect(push?.patch.duration).toBe(seconds({ value: 10 }));
		expect(push?.patch.trimStart).toBe(ZERO_MEDIA_TIME);
		expect(push?.patch.trimEnd).toBe(ZERO_MEDIA_TIME);
	});

	test("pushes a chain of several neighbors by the same overflow", () => {
		const member = buildStaticMember({
			startTime: ZERO_MEDIA_TIME,
			duration: seconds({ value: 5 }),
			rightNeighborBound: seconds({ value: 5 }),
			rightPushChain: [
				buildPushTarget({
					elementId: "neighbor-1",
					startTime: seconds({ value: 5 }),
					duration: seconds({ value: 10 }),
				}),
				buildPushTarget({
					elementId: "neighbor-2",
					startTime: seconds({ value: 15 }),
					duration: seconds({ value: 5 }),
				}),
				buildPushTarget({
					elementId: "neighbor-3",
					startTime: seconds({ value: 25 }),
					duration: seconds({ value: 2 }),
				}),
			],
		});

		const result = computeGroupResize({
			members: [member],
			side: "right",
			deltaTime: seconds({ value: 4 }),
			fps: FPS,
		});

		expect(result.deltaTime).toBe(seconds({ value: 4 }));
		expect(result.updates).toHaveLength(4);
		expect(
			findUpdate({ updates: result.updates, elementId: "neighbor-1" })?.patch
				.startTime,
		).toBe(seconds({ value: 9 }));
		expect(
			findUpdate({ updates: result.updates, elementId: "neighbor-2" })?.patch
				.startTime,
		).toBe(seconds({ value: 19 }));
		expect(
			findUpdate({ updates: result.updates, elementId: "neighbor-3" })?.patch
				.startTime,
		).toBe(seconds({ value: 29 }));
	});

	test("push stops at the end of the track", () => {
		const member = buildStaticMember({
			startTime: ZERO_MEDIA_TIME,
			duration: seconds({ value: 5 }),
			rightNeighborBound: null,
			rightPushChain: [],
		});

		const result = computeGroupResize({
			members: [member],
			side: "right",
			deltaTime: seconds({ value: 3 }),
			fps: FPS,
		});

		expect(result.deltaTime).toBe(seconds({ value: 3 }));
		expect(result.updates).toHaveLength(1);
		expect(
			findUpdate({ updates: result.updates, elementId: "element-1" })?.patch
				.duration,
		).toBe(seconds({ value: 8 }));
	});

	test("video element clamps at its right neighbor instead of pushing", () => {
		const member = buildVideoMember({
			startTime: ZERO_MEDIA_TIME,
			duration: seconds({ value: 5 }),
			sourceDuration: seconds({ value: 6 }),
			rightNeighborBound: seconds({ value: 5 }),
		});

		const result = computeGroupResize({
			members: [member],
			side: "right",
			deltaTime: seconds({ value: 10 }),
			fps: FPS,
		});

		expect(result.deltaTime).toBe(ZERO_MEDIA_TIME);
		expect(result.updates).toHaveLength(1);
		expect(
			findUpdate({ updates: result.updates, elementId: "element-1" })?.patch
				.duration,
		).toBe(seconds({ value: 5 }));
	});

	test("video element clamps at its source extent when there is no neighbor", () => {
		const member = buildVideoMember({
			startTime: ZERO_MEDIA_TIME,
			duration: seconds({ value: 5 }),
			sourceDuration: seconds({ value: 6 }),
			rightNeighborBound: null,
		});

		const result = computeGroupResize({
			members: [member],
			side: "right",
			deltaTime: seconds({ value: 10 }),
			fps: FPS,
		});

		expect(result.deltaTime).toBe(seconds({ value: 1 }));
		expect(result.updates).toHaveLength(1);
		expect(
			findUpdate({ updates: result.updates, elementId: "element-1" })?.patch
				.duration,
		).toBe(seconds({ value: 6 }));
	});

	test("static element without a push chain keeps the old neighbor clamp", () => {
		const member = buildStaticMember({
			startTime: ZERO_MEDIA_TIME,
			duration: seconds({ value: 5 }),
			rightNeighborBound: seconds({ value: 5 }),
		});

		const result = computeGroupResize({
			members: [member],
			side: "right",
			deltaTime: seconds({ value: 3 }),
			fps: FPS,
		});

		expect(result.deltaTime).toBe(ZERO_MEDIA_TIME);
		expect(result.updates).toHaveLength(1);
	});

	test("shortening still respects the one-frame minimum duration", () => {
		const oneFrame = mediaTime({
			ticks: Math.round(TICKS_PER_SECOND / 30),
		});
		const member = buildStaticMember({
			startTime: ZERO_MEDIA_TIME,
			duration: seconds({ value: 5 }),
			rightNeighborBound: null,
			rightPushChain: [],
		});

		const result = computeGroupResize({
			members: [member],
			side: "right",
			deltaTime: seconds({ value: -10 }),
			fps: FPS,
		});

		expect(result.updates).toHaveLength(1);
		expect(
			findUpdate({ updates: result.updates, elementId: "element-1" })?.patch
				.duration,
		).toBe(oneFrame);
	});

	test("left-edge resize still clamps at the left neighbor and pushes nothing", () => {
		const member = buildStaticMember({
			startTime: seconds({ value: 5 }),
			duration: seconds({ value: 5 }),
			leftNeighborBound: seconds({ value: 2 }),
			rightNeighborBound: null,
			rightPushChain: [
				buildPushTarget({
					elementId: "neighbor-1",
					startTime: seconds({ value: 10 }),
					duration: seconds({ value: 10 }),
				}),
			],
		});

		const result = computeGroupResize({
			members: [member],
			side: "left",
			deltaTime: seconds({ value: -10 }),
			fps: FPS,
		});

		expect(result.deltaTime).toBe(seconds({ value: -3 }));
		expect(result.updates).toHaveLength(1);
		const update = findUpdate({ updates: result.updates, elementId: "element-1" });
		expect(update?.patch.startTime).toBe(seconds({ value: 2 }));
		expect(update?.patch.duration).toBe(seconds({ value: 8 }));
	});
});

const IMAGE_DURATION = seconds({ value: 5 });
const VIDEO_DURATION = seconds({ value: 10 });
const TAIL_DURATION = seconds({ value: 5 });
const PUSH_DELTA = seconds({ value: 3 });

function buildImageElement(): ImageElement {
	return {
		id: "image-1",
		type: "image",
		name: "image-1",
		startTime: ZERO_MEDIA_TIME,
		duration: IMAGE_DURATION,
		trimStart: ZERO_MEDIA_TIME,
		trimEnd: ZERO_MEDIA_TIME,
		mediaId: "media-image-1",
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

function buildVideoElement({
	id,
	startTime,
	duration,
}: {
	id: string;
	startTime: MediaTime;
	duration: MediaTime;
}): VideoElement {
	return {
		id,
		type: "video",
		name: id,
		startTime,
		duration,
		trimStart: ZERO_MEDIA_TIME,
		trimEnd: ZERO_MEDIA_TIME,
		sourceDuration: duration,
		mediaId: `media-${id}`,
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

function seedScene(): { editor: EditorCore; trackId: string } {
	const editor = EditorCore.getInstance();
	const scene = buildDefaultScene({ name: "Main scene", isMain: true });
	scene.tracks.main.elements = [
		buildImageElement(),
		buildVideoElement({
			id: "video-1",
			startTime: IMAGE_DURATION,
			duration: VIDEO_DURATION,
		}),
		buildVideoElement({
			id: "video-2",
			startTime: addMediaTime({ a: IMAGE_DURATION, b: VIDEO_DURATION }),
			duration: TAIL_DURATION,
		}),
	];
	editor.project.setActiveProject({
		project: {
			metadata: {
				id: "project-1",
				name: "Test project",
				duration: addMediaTime({
					a: IMAGE_DURATION,
					b: addMediaTime({ a: VIDEO_DURATION, b: TAIL_DURATION }),
				}),
				createdAt: new Date("2026-01-01T00:00:00Z"),
				updatedAt: new Date("2026-01-01T00:00:00Z"),
			},
			scenes: [scene],
			currentSceneId: scene.id,
			settings: {
				fps: FPS,
				canvasSize: { width: 1920, height: 1080 },
				background: { type: "color", color: "#000000" },
			},
			version: 1,
		},
	});
	editor.scenes.setScenes({ scenes: [scene], activeSceneId: scene.id });
	return { editor, trackId: scene.tracks.main.id };
}

function getElementById({
	editor,
	elementId,
}: {
	editor: EditorCore;
	elementId: string;
}): TimelineElement | undefined {
	return editor.scenes
		.getActiveScene()
		.tracks.main.elements.find((element) => element.id === elementId);
}

function commitResizeResult({
	editor,
	trackId,
}: {
	editor: EditorCore;
	trackId: string;
}): void {
	const members = buildResizeMembers({
		tracks: editor.scenes.getActiveScene().tracks,
		selectedElements: [{ trackId, elementId: "image-1" }],
	});
	const result = computeGroupResize({
		members,
		side: "right",
		deltaTime: PUSH_DELTA,
		fps: FPS,
	});
	editor.command.execute({
		command: new UpdateElementsCommand({
			updates: result.updates.map(
				({ trackId: updateTrackId, elementId, patch }) => ({
					trackId: updateTrackId,
					elementId,
					patch: patch as Partial<TimelineElement>,
				}),
			),
		}),
	});
}

function expectPushedLayout({ editor }: { editor: EditorCore }): void {
	expect(getElementById({ editor, elementId: "image-1" })?.duration).toBe(
		addMediaTime({ a: IMAGE_DURATION, b: PUSH_DELTA }),
	);
	expect(getElementById({ editor, elementId: "video-1" })?.startTime).toBe(
		addMediaTime({ a: IMAGE_DURATION, b: PUSH_DELTA }),
	);
	expect(getElementById({ editor, elementId: "video-2" })?.startTime).toBe(
		addMediaTime({
			a: addMediaTime({ a: IMAGE_DURATION, b: VIDEO_DURATION }),
			b: PUSH_DELTA,
		}),
	);
}

function expectOriginalLayout({ editor }: { editor: EditorCore }): void {
	expect(getElementById({ editor, elementId: "image-1" })?.duration).toBe(
		IMAGE_DURATION,
	);
	expect(getElementById({ editor, elementId: "video-1" })?.startTime).toBe(
		IMAGE_DURATION,
	);
	expect(getElementById({ editor, elementId: "video-2" })?.startTime).toBe(
		addMediaTime({ a: IMAGE_DURATION, b: VIDEO_DURATION }),
	);
}

describe("ripple push commit", () => {
	beforeEach(() => {
		EditorCore.reset();
	});

	test("resize and push land in one undoable step", () => {
		const { editor, trackId } = seedScene();

		commitResizeResult({ editor, trackId });

		expectPushedLayout({ editor });
	});

	test("undo restores the original positions", () => {
		const { editor, trackId } = seedScene();

		commitResizeResult({ editor, trackId });
		editor.command.undo();

		expectOriginalLayout({ editor });
	});

	test("redo after undo re-applies the resize and push", () => {
		const { editor, trackId } = seedScene();

		commitResizeResult({ editor, trackId });
		editor.command.undo();
		editor.command.redo();

		expectPushedLayout({ editor });
	});
});
