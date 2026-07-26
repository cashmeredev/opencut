import { describe, expect, test } from "bun:test";
import type { ProjectRecord } from "@/services/storage/migrations/transformers/types";
import {
	collectReferencedMediaIds,
	remapProjectMediaIds,
	resolveImportedProjectName,
} from "../remap";

function buildProject(): ProjectRecord {
	return {
		version: 31,
		metadata: { id: "project-1", name: "Demo" },
		currentSceneId: "scene-1",
		scenes: [
			{
				id: "scene-1",
				tracks: {
					main: {
						id: "track-main",
						elements: [
							{ id: "el-video", type: "video", mediaId: "media-video" },
							{ id: "el-image", type: "image", mediaId: "media-image" },
						],
					},
					overlay: [
						{
							id: "track-text",
							elements: [{ id: "el-text", type: "text", content: "hello" }],
						},
					],
					audio: [
						{
							id: "track-audio",
							elements: [
								{
									id: "el-upload",
									type: "audio",
									sourceType: "upload",
									mediaId: "media-audio",
								},
								{
									id: "el-library",
									type: "audio",
									sourceType: "library",
									sourceUrl: "https://example.com/sound.mp3",
								},
								{
									id: "el-shared",
									type: "audio",
									sourceType: "upload",
									mediaId: "media-video",
								},
							],
						},
					],
				},
			},
			{
				id: "scene-2",
				tracks: {
					main: {
						id: "track-main-2",
						elements: [
							{ id: "el-video-2", type: "video", mediaId: "media-video-2" },
						],
					},
					overlay: [],
					audio: [],
				},
			},
		],
	};
}

describe("collectReferencedMediaIds", () => {
	test("collects media ids across scenes and track groups, deduplicated", () => {
		const ids = collectReferencedMediaIds({ project: buildProject() });
		expect(ids.sort()).toEqual([
			"media-audio",
			"media-image",
			"media-video",
			"media-video-2",
		]);
	});

	test("tolerates malformed shapes", () => {
		expect(collectReferencedMediaIds({ project: null })).toEqual([]);
		expect(collectReferencedMediaIds({ project: { scenes: "nope" } })).toEqual(
			[],
		);
		expect(
			collectReferencedMediaIds({
				project: { scenes: [{ tracks: { main: { elements: "nope" } } }] },
			}),
		).toEqual([]);
	});
});

describe("remapProjectMediaIds", () => {
	test("replaces mapped ids only, across every track group", () => {
		const project = buildProject();
		const idMap = new Map([
			["media-video", "new-video"],
			["media-image", "new-image"],
			["media-audio", "new-audio"],
		]);

		const remapped = remapProjectMediaIds({ project, idMap });
		const collected = collectReferencedMediaIds({ project: remapped });
		expect(collected.sort()).toEqual([
			"media-video-2",
			"new-audio",
			"new-image",
			"new-video",
		]);

		// The shared media id is remapped in every referencing element.
		const scenes = remapped.scenes;
		expect(JSON.stringify(scenes)).not.toContain("media-video\"");
		expect(JSON.stringify(scenes)).toContain("new-video");
	});

	test("does not mutate the input document", () => {
		const project = buildProject();
		const before = JSON.stringify(project);
		remapProjectMediaIds({
			project,
			idMap: new Map([["media-video", "new-video"]]),
		});
		expect(JSON.stringify(project)).toBe(before);
	});

	test("returns the input unchanged when the map is empty", () => {
		const project = buildProject();
		expect(remapProjectMediaIds({ project, idMap: new Map() })).toBe(project);
	});

	test("is fully reversible with the inverse map", () => {
		const project = buildProject();
		const idMap = new Map([
			["media-video", "new-video"],
			["media-image", "new-image"],
			["media-audio", "new-audio"],
			["media-video-2", "new-video-2"],
		]);
		const inverse = new Map([...idMap].map(([from, to]) => [to, from]));

		const remapped = remapProjectMediaIds({ project, idMap });
		const restored = remapProjectMediaIds({ project: remapped, idMap: inverse });
		expect(restored).toEqual(project);
	});
});

describe("resolveImportedProjectName", () => {
	test("keeps the name when there is no collision", () => {
		expect(
			resolveImportedProjectName({ name: "Demo", existingNames: ["Other"] }),
		).toBe("Demo");
	});

	test("suffixes the name when it collides", () => {
		expect(
			resolveImportedProjectName({ name: "Demo", existingNames: ["Demo"] }),
		).toBe("Demo (imported)");
	});
});
