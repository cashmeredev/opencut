import { describe, expect, test } from "bun:test";
import {
	CURRENT_PROJECT_VERSION,
	migrations,
} from "@/services/storage/migrations";
import type { ProjectRecord } from "@/services/storage/migrations/transformers/types";
import { buildProjectArchive, parseProjectArchive } from "../container";
import { migrateProjectRecord } from "../migrate";
import {
	collectReferencedMediaIds,
	remapProjectMediaIds,
} from "../remap";

/**
 * The transfer contract, proven at the pure-function level:
 * export -> import keeps the project document deep-equal (modulo the media
 * id remap) and every media file byte-identical.
 */

function buildCurrentProject(): ProjectRecord {
	return {
		version: CURRENT_PROJECT_VERSION,
		metadata: {
			id: "project-1",
			name: "Round Trip",
			thumbnail: "data:image/png;base64,thumb",
			duration: 240_000,
			createdAt: "2024-06-01T10:00:00.000Z",
			updatedAt: "2024-06-01T14:00:00.000Z",
		},
		currentSceneId: "scene-1",
		settings: {
			fps: 30,
			canvasSize: { width: 1920, height: 1080 },
			canvasSizeMode: "preset",
			lastCustomCanvasSize: null,
			originalCanvasSize: null,
			background: { type: "color", color: "#000000" },
		},
		timelineViewState: { zoomLevel: 1, scrollLeft: 0, playheadTime: 0 },
		scenes: [
			{
				id: "scene-1",
				name: "Main scene",
				isMain: true,
				bookmarks: [{ time: 120_000, note: "mid", color: "#fff" }],
				createdAt: "2024-06-01T10:00:00.000Z",
				updatedAt: "2024-06-01T14:00:00.000Z",
				tracks: {
					main: {
						id: "track-main",
						type: "video",
						name: "Main",
						muted: false,
						hidden: false,
						elements: [
							{
								id: "el-video",
								type: "video",
								mediaId: "media-video",
								name: "clip.mp4",
								duration: 240_000,
								startTime: 0,
								trimStart: 0,
								trimEnd: 0,
								params: { opacity: 1 },
							},
							{
								id: "el-image",
								type: "image",
								mediaId: "media-image",
								name: "still.png",
								duration: 120_000,
								startTime: 240_000,
								trimStart: 0,
								trimEnd: 0,
								params: {},
							},
						],
					},
					overlay: [
						{
							id: "track-text",
							type: "text",
							name: "Text",
							hidden: false,
							elements: [
								{
									id: "el-text",
									type: "text",
									name: "Title",
									duration: 60_000,
									startTime: 0,
									trimStart: 0,
									trimEnd: 0,
									params: { content: "Hello" },
								},
							],
						},
					],
					audio: [
						{
							id: "track-audio",
							type: "audio",
							name: "Audio",
							muted: false,
							elements: [
								{
									id: "el-audio",
									type: "audio",
									sourceType: "upload",
									mediaId: "media-audio",
									name: "song.mp3",
									duration: 240_000,
									startTime: 0,
									trimStart: 0,
									trimEnd: 0,
									params: { volume: 0.8 },
								},
							],
						},
					],
				},
			},
		],
	};
}

function buildMedia() {
	return [
		{
			metadata: {
				id: "media-video",
				name: "clip.mp4",
				mediaType: "video" as const,
				fileType: "video/mp4",
				lastModified: 1_700_000_000_000,
				width: 1920,
				height: 1080,
				duration: 2,
			},
			data: new Uint8Array([0, 1, 2, 3, 250, 251, 252, 253, 254, 255]),
		},
		{
			metadata: {
				id: "media-image",
				name: "still.png",
				mediaType: "image" as const,
				fileType: "image/png",
				lastModified: 1_700_000_000_001,
				width: 800,
				height: 600,
				thumbnailUrl: "data:image/png;base64,still",
			},
			data: new Uint8Array([137, 80, 78, 71]),
		},
		{
			metadata: {
				id: "media-audio",
				name: "song.mp3",
				mediaType: "audio" as const,
				fileType: "audio/mpeg",
				lastModified: 1_700_000_000_002,
				duration: 2,
			},
			data: new Uint8Array([255, 251, 144, 0]),
		},
	];
}

describe("project transfer round trip", () => {
	test("export -> import keeps the document deep-equal and media byte-identical", async () => {
		const project = buildCurrentProject();
		const media = buildMedia();

		// "Export": pack the serialized document plus original media bytes.
		const archive = buildProjectArchive({
			projectJson: JSON.stringify(project),
			media,
		});

		// "Import": parse, run the same migration pipeline, remap media ids.
		const parsed = parseProjectArchive({ archive });
		const migrated = await migrateProjectRecord({
			project: parsed.project,
			migrations,
		});
		expect(migrated.complete).toBe(true);

		const referenced = collectReferencedMediaIds({ project });
		expect(referenced.sort()).toEqual(
			media.map(({ metadata }) => metadata.id).sort(),
		);

		const idMap = new Map(
			parsed.media.map((entry) => [entry.id, `imported-${entry.id}`]),
		);
		const imported = remapProjectMediaIds({ project: migrated.project, idMap });

		// Every media reference points at a fresh id, nothing else changed:
		// remapping back with the inverse map restores the original document.
		expect(collectReferencedMediaIds({ project: imported }).sort()).toEqual(
			[...idMap.values()].sort(),
		);
		const restored = remapProjectMediaIds({
			project: imported,
			idMap: new Map([...idMap].map(([from, to]) => [to, from])),
		});
		expect(restored).toEqual(project);

		// Media comes back byte-identical with its full metadata.
		for (const { metadata, data } of media) {
			const entry = parsed.media.find(({ id }) => id === metadata.id);
			expect(entry).toBeDefined();
			expect(entry).toMatchObject(metadata);
			expect([...parsed.readMediaData({ entry: entry?.entry ?? "" })]).toEqual([
				...data,
			]);
		}
	});
});
