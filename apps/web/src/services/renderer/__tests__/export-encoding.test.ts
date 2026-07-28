import { describe, expect, test } from "bun:test";
import { computeVideoBitrate } from "../export-encoding";

describe("computeVideoBitrate", () => {
	test("scales linearly with frame rate at the same resolution and quality", () => {
		const at30 = computeVideoBitrate({
			width: 1920,
			height: 1080,
			fps: 30,
			quality: "high",
		});
		const at60 = computeVideoBitrate({
			width: 1920,
			height: 1080,
			fps: 60,
			quality: "high",
		});

		expect(at60).toBe(at30 * 2);
	});

	test("applies the bits-per-pixel ladder per quality tier", () => {
		const params = { width: 1920, height: 1080, fps: 30 };

		expect(computeVideoBitrate({ ...params, quality: "low" })).toBe(3_110_400);
		expect(computeVideoBitrate({ ...params, quality: "medium" })).toBe(
			6_220_800,
		);
		expect(computeVideoBitrate({ ...params, quality: "high" })).toBe(
			12_441_600,
		);
		expect(computeVideoBitrate({ ...params, quality: "very_high" })).toBe(
			24_883_200,
		);
	});

	test("increases monotonically across quality tiers", () => {
		const params = { width: 1280, height: 720, fps: 30 };
		const bitrates = (["low", "medium", "high", "very_high"] as const).map(
			(quality) => computeVideoBitrate({ ...params, quality }),
		);

		for (let i = 1; i < bitrates.length; i++) {
			expect(bitrates[i]).toBeGreaterThan(bitrates[i - 1]);
		}
	});

	test("returns a positive integer for fractional frame rates", () => {
		const bitrate = computeVideoBitrate({
			width: 1920,
			height: 1080,
			fps: 29.97,
			quality: "high",
		});

		expect(bitrate).toBe(12_429_158);
		expect(Number.isInteger(bitrate)).toBe(true);
		expect(bitrate).toBeGreaterThan(0);
	});
});
