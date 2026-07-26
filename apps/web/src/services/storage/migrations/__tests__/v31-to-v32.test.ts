import { describe, expect, test } from "bun:test";
import { transformProjectV31ToV32 } from "../transformers/v31-to-v32";
import { asRecord, asRecordArray } from "./helpers";

describe("V31 to V32 Migration", () => {
	test("stamps a v31 project as v32 without changing its content", () => {
		const result = transformProjectV31ToV32({
			project: {
				id: "project-v31",
				version: 31,
				metadata: { id: "project-v31", name: "My project" },
				scenes: [
					{
						id: "scene-1",
						tracks: {
							main: {
								id: "track-1",
								type: "video",
								elements: [{ id: "elem-1", type: "video", mediaId: "media-1" }],
							},
							overlay: [],
							audio: [],
						},
					},
				],
			},
		});

		expect(result.skipped).toBe(false);
		expect(result.project.version).toBe(32);
		const scene = asRecordArray(result.project.scenes)[0];
		const main = asRecord(asRecord(scene.tracks).main);
		const element = asRecordArray(main.elements)[0];
		expect(element).toMatchObject({
			id: "elem-1",
			type: "video",
			mediaId: "media-1",
		});
		expect(asRecord(result.project.metadata).name).toBe("My project");
	});

	test("skips a project that is already v32", () => {
		const project = { id: "p1", version: 32, scenes: [] };
		const result = transformProjectV31ToV32({ project });
		expect(result.skipped).toBe(true);
		expect(result.reason).toBe("already v32");
		expect(result.project).toBe(project);
	});

	test("skips a project that is not v31", () => {
		const project = { id: "p1", version: 30, scenes: [] };
		const result = transformProjectV31ToV32({ project });
		expect(result.skipped).toBe(true);
		expect(result.reason).toBe("not v31");
		expect(result.project).toBe(project);
	});

	test("skips a project without an id", () => {
		const project = { version: 31, scenes: [] };
		const result = transformProjectV31ToV32({ project });
		expect(result.skipped).toBe(true);
		expect(result.reason).toBe("no project id");
		expect(result.project).toBe(project);
	});
});
