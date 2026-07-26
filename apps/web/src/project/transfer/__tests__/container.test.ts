import { describe, expect, test } from "bun:test";
import { zipSync, strToU8 } from "fflate";
import {
	MEDIA_MANIFEST_ENTRY,
	PROJECT_JSON_ENTRY,
	ProjectArchiveError,
	buildMediaEntryName,
	buildProjectArchive,
	parseProjectArchive,
	sanitizeFileName,
} from "../container";

const projectJson = JSON.stringify({
	version: 31,
	metadata: {
		id: "project-1",
		name: "Demo",
		createdAt: "2024-06-01T10:00:00.000Z",
		updatedAt: "2024-06-01T14:00:00.000Z",
	},
	scenes: [],
	currentSceneId: "scene-1",
	settings: { fps: 30 },
});

function buildMediaInputs() {
	return [
		{
			metadata: {
				id: "media-1",
				name: "clip one.mp4",
				mediaType: "video" as const,
				fileType: "video/mp4",
				lastModified: 1_700_000_000_000,
				width: 1920,
				height: 1080,
				duration: 3.5,
			},
			data: new Uint8Array([1, 2, 3, 4, 5]),
		},
		{
			metadata: {
				id: "media-2",
				name: "picturé/odd:name.png",
				mediaType: "image" as const,
				fileType: "image/png",
				lastModified: 1_700_000_000_001,
				thumbnailUrl: "data:image/png;base64,abc",
			},
			data: new Uint8Array([9, 8, 7]),
		},
	];
}

describe("sanitizeFileName", () => {
	test("replaces unsafe characters and keeps the result stable", () => {
		expect(sanitizeFileName({ name: "my clip.mp4" })).toBe("my_clip.mp4");
		expect(sanitizeFileName({ name: "a/b\\c:d" })).toBe("a_b_c_d");
		expect(sanitizeFileName({ name: "picturé.png" })).toBe("pictur_.png");
	});

	test("falls back for empty or dot-only names", () => {
		expect(sanitizeFileName({ name: "   " })).toBe("file");
		expect(sanitizeFileName({ name: "..." })).toBe("file");
	});
});

describe("buildMediaEntryName", () => {
	test("prefixes the asset id and sanitizes the name", () => {
		expect(buildMediaEntryName({ assetId: "abc", name: "my clip.mp4" })).toBe(
			"media/abc-my_clip.mp4",
		);
	});
});

describe("buildProjectArchive + parseProjectArchive", () => {
	test("round-trips project document, manifest, and media bytes", () => {
		const media = buildMediaInputs();
		const archive = buildProjectArchive({ projectJson, media });
		const parsed = parseProjectArchive({ archive });

		expect(JSON.parse(JSON.stringify(parsed.project))).toEqual(
			JSON.parse(projectJson),
		);

		expect(parsed.media).toHaveLength(2);
		const [first, second] = parsed.media;
		expect(first).toMatchObject({
			id: "media-1",
			name: "clip one.mp4",
			mediaType: "video",
			fileType: "video/mp4",
			lastModified: 1_700_000_000_000,
			width: 1920,
			height: 1080,
			duration: 3.5,
			entry: "media/media-1-clip_one.mp4",
		});
		expect(second).toMatchObject({
			id: "media-2",
			name: "picturé/odd:name.png",
			mediaType: "image",
			fileType: "image/png",
			thumbnailUrl: "data:image/png;base64,abc",
		});

		expect([...parsed.readMediaData({ entry: first.entry })]).toEqual([
			1, 2, 3, 4, 5,
		]);
		expect([...parsed.readMediaData({ entry: second.entry })]).toEqual([9, 8, 7]);
	});

	test("supports archives without media", () => {
		const archive = buildProjectArchive({ projectJson, media: [] });
		const parsed = parseProjectArchive({ archive });
		expect(parsed.media).toEqual([]);
	});
});

describe("parseProjectArchive validation", () => {
	test("rejects non-zip data", () => {
		expect(() =>
			parseProjectArchive({ archive: new Uint8Array([1, 2, 3]) }),
		).toThrow(ProjectArchiveError);
	});

	test("rejects archives without project.json", () => {
		const archive = zipSync({ "other.txt": strToU8("hello") });
		expect(() => parseProjectArchive({ archive })).toThrow(/project\.json/);
	});

	test("rejects invalid project.json", () => {
		const archive = zipSync({
			[PROJECT_JSON_ENTRY]: strToU8("not json{"),
		});
		expect(() => parseProjectArchive({ archive })).toThrow(/not valid JSON/);
	});

	test("rejects non-object project.json", () => {
		const archive = zipSync({ [PROJECT_JSON_ENTRY]: strToU8("[1,2,3]") });
		expect(() => parseProjectArchive({ archive })).toThrow(/not a project/);
	});

	test("rejects media entries without a manifest", () => {
		const archive = zipSync({
			[PROJECT_JSON_ENTRY]: strToU8(projectJson),
			"media/media-1-clip.mp4": new Uint8Array([1]),
		});
		expect(() => parseProjectArchive({ archive })).toThrow(/manifest/);
	});

	test("rejects manifest entries whose media file is missing", () => {
		const archive = zipSync({
			[PROJECT_JSON_ENTRY]: strToU8(projectJson),
			[MEDIA_MANIFEST_ENTRY]: strToU8(
				JSON.stringify({
					version: 1,
					media: [
						{
							id: "media-1",
							name: "clip.mp4",
							entry: "media/media-1-clip.mp4",
							mediaType: "video",
							fileType: "video/mp4",
							lastModified: 1,
						},
					],
				}),
			),
		});
		expect(() => parseProjectArchive({ archive })).toThrow(/is missing/);
	});

	test("rejects manifest entries with paths outside media/", () => {
		const archive = zipSync({
			[PROJECT_JSON_ENTRY]: strToU8(projectJson),
			[MEDIA_MANIFEST_ENTRY]: strToU8(
				JSON.stringify({
					version: 1,
					media: [
						{
							id: "media-1",
							name: "evil",
							entry: "../evil",
							mediaType: "video",
							fileType: "video/mp4",
							lastModified: 1,
						},
					],
				}),
			),
			"media/placeholder": new Uint8Array([1]),
		});
		expect(() => parseProjectArchive({ archive })).toThrow(/invalid file path/);
	});

	test("readMediaData throws for unknown entries", () => {
		const archive = buildProjectArchive({ projectJson, media: [] });
		const parsed = parseProjectArchive({ archive });
		expect(() => parsed.readMediaData({ entry: "media/nope" })).toThrow(
			ProjectArchiveError,
		);
	});
});
