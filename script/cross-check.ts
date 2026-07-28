import { readFileSync, writeFileSync } from "node:fs";
import {
	buildProjectArchive,
	parseProjectArchive,
} from "../apps/web/src/project/transfer/container";

const direction = process.argv[2];

if (direction === "build") {
	const projectJson = JSON.stringify({
		version: 7,
		metadata: { id: "web-built" },
		nested: { value: 42 },
	});
	const archive = buildProjectArchive({
		projectJson,
		media: [
			{
				metadata: {
					id: "web-asset",
					name: "web clip.mov",
					mediaType: "video",
					fileType: "video/quicktime",
					lastModified: 1_753_000_000_000,
					width: 1280,
					height: 720,
					duration: 3.25,
				},
				data: new Uint8Array([9, 8, 7, 6]),
			},
		],
	});
	writeFileSync("/tmp/web-built.ocp", archive);
	console.log("web-built.ocp written");
} else if (direction === "parse") {
	const archive = readFileSync("/tmp/rust-built.ocp");
	const parsed = parseProjectArchive({ archive });
	console.log(
		JSON.stringify({
			project: parsed.project,
			media: parsed.media,
			firstBytes: Array.from(
				parsed.readMediaData({ entry: parsed.media[0]?.entry ?? "" }),
			),
		}),
	);
} else {
	throw new Error("usage: bun cross-check.ts build|parse");
}
