import { describe, expect, test } from "bun:test";
import {
	buildThumbnailTileKey,
	FILMSTRIP_MAX_INTERVAL_SECONDS,
	FILMSTRIP_MIN_INTERVAL_SECONDS,
	getFilmstripInterval,
	getFilmstripSlotCount,
	getFilmstripSlotSourceTime,
	getVisibleSlotRange,
	MEDIA_TILE_ASPECT_RATIO,
} from "../tiling";

const TRACK_HEIGHT_PX = 65;
const MIN_TILE_WIDTH_PX = TRACK_HEIGHT_PX * MEDIA_TILE_ASPECT_RATIO;

describe("getFilmstripInterval", () => {
	test("snaps to the nearest power of two seconds", () => {
		expect(
			getFilmstripInterval({
				pixelsPerSecond: 100,
				minTileWidthPx: MIN_TILE_WIDTH_PX,
			}),
		).toBe(1);
		expect(
			getFilmstripInterval({
				pixelsPerSecond: 50,
				minTileWidthPx: MIN_TILE_WIDTH_PX,
			}),
		).toBe(2);
		expect(
			getFilmstripInterval({
				pixelsPerSecond: 200,
				minTileWidthPx: MIN_TILE_WIDTH_PX,
			}),
		).toBe(0.5);
	});

	test("keeps tile width near the minimum across zoom levels", () => {
		for (const pixelsPerSecond of [5, 12, 25, 40, 80, 160, 320, 640]) {
			const interval = getFilmstripInterval({
				pixelsPerSecond,
				minTileWidthPx: MIN_TILE_WIDTH_PX,
			});
			const tileWidthPx = interval * pixelsPerSecond;
			expect(tileWidthPx).toBeGreaterThanOrEqual(MIN_TILE_WIDTH_PX * 0.7);
			expect(tileWidthPx).toBeLessThanOrEqual(MIN_TILE_WIDTH_PX * 1.45);
		}
	});

	test("is stable under tiny zoom changes", () => {
		const coarse = getFilmstripInterval({
			pixelsPerSecond: 100,
			minTileWidthPx: MIN_TILE_WIDTH_PX,
		});
		const nudged = getFilmstripInterval({
			pixelsPerSecond: 110,
			minTileWidthPx: MIN_TILE_WIDTH_PX,
		});
		expect(nudged).toBe(coarse);
	});

	test("clamps the interval at extreme zoom levels", () => {
		const zoomedIn = getFilmstripInterval({
			pixelsPerSecond: 5000,
			minTileWidthPx: MIN_TILE_WIDTH_PX,
		});
		expect(zoomedIn).toBeGreaterThanOrEqual(FILMSTRIP_MIN_INTERVAL_SECONDS);

		const zoomedOut = getFilmstripInterval({
			pixelsPerSecond: 5,
			minTileWidthPx: MIN_TILE_WIDTH_PX,
		});
		expect(zoomedOut).toBeLessThanOrEqual(FILMSTRIP_MAX_INTERVAL_SECONDS);
	});

	test("falls back to the max interval for invalid zoom", () => {
		expect(
			getFilmstripInterval({
				pixelsPerSecond: 0,
				minTileWidthPx: MIN_TILE_WIDTH_PX,
			}),
		).toBe(FILMSTRIP_MAX_INTERVAL_SECONDS);
		expect(
			getFilmstripInterval({
				pixelsPerSecond: Number.NaN,
				minTileWidthPx: MIN_TILE_WIDTH_PX,
			}),
		).toBe(FILMSTRIP_MAX_INTERVAL_SECONDS);
	});
});

describe("getFilmstripSlotCount", () => {
	test("covers the clip duration with exact tiles", () => {
		expect(getFilmstripSlotCount({ clipDurationSec: 10, interval: 2 })).toBe(5);
		expect(getFilmstripSlotCount({ clipDurationSec: 10.5, interval: 2 })).toBe(
			6,
		);
	});

	test("does not add a trailing tile on exact multiples", () => {
		expect(getFilmstripSlotCount({ clipDurationSec: 4, interval: 2 })).toBe(2);
	});

	test("returns zero for empty clips", () => {
		expect(getFilmstripSlotCount({ clipDurationSec: 0, interval: 2 })).toBe(0);
		expect(getFilmstripSlotCount({ clipDurationSec: -3, interval: 2 })).toBe(0);
	});
});

describe("getVisibleSlotRange", () => {
	test("returns the slots intersecting the visible window plus overscan", () => {
		const range = getVisibleSlotRange({
			slotCount: 100,
			tileWidthPx: 100,
			visibleLeftPx: 500,
			visibleRightPx: 1000,
			overscanSlots: 2,
		});
		expect(range).toEqual({ firstSlot: 3, lastSlot: 11 });
	});

	test("clamps to the slot bounds", () => {
		const range = getVisibleSlotRange({
			slotCount: 8,
			tileWidthPx: 100,
			visibleLeftPx: 0,
			visibleRightPx: 1000,
			overscanSlots: 2,
		});
		expect(range).toEqual({ firstSlot: 0, lastSlot: 7 });
	});

	test("returns an empty range when nothing is visible", () => {
		const range = getVisibleSlotRange({
			slotCount: 10,
			tileWidthPx: 100,
			visibleLeftPx: 500,
			visibleRightPx: 500,
			overscanSlots: 2,
		});
		expect(range.lastSlot).toBeLessThan(range.firstSlot);
	});
});

describe("getFilmstripSlotSourceTime", () => {
	test("offsets by the trim start", () => {
		expect(
			getFilmstripSlotSourceTime({
				slot: 2,
				interval: 1,
				trimStartSec: 5,
				rate: 1,
			}),
		).toBe(7);
	});

	test("scales clip time by the retime rate", () => {
		expect(
			getFilmstripSlotSourceTime({
				slot: 2,
				interval: 1,
				trimStartSec: 5,
				rate: 2,
			}),
		).toBe(9);
	});

	test("clamps to the end of the asset", () => {
		const clamped = getFilmstripSlotSourceTime({
			slot: 100,
			interval: 1,
			trimStartSec: 0,
			rate: 1,
			assetDurationSec: 10,
		});
		expect(clamped).toBeLessThan(10);
		expect(clamped).toBeGreaterThan(9);
	});

	test("never returns negative times", () => {
		expect(
			getFilmstripSlotSourceTime({
				slot: 0,
				interval: 1,
				trimStartSec: -4,
				rate: 1,
			}),
		).toBe(0);
	});
});

describe("buildThumbnailTileKey", () => {
	test("changes per media, size, and quantized time", () => {
		const base = buildThumbnailTileKey({
			mediaId: "media-1",
			time: 1,
			width: 232,
			height: 130,
		});
		expect(base).not.toBe(
			buildThumbnailTileKey({
				mediaId: "media-2",
				time: 1,
				width: 232,
				height: 130,
			}),
		);
		expect(base).not.toBe(
			buildThumbnailTileKey({
				mediaId: "media-1",
				time: 2,
				width: 232,
				height: 130,
			}),
		);
		expect(base).not.toBe(
			buildThumbnailTileKey({
				mediaId: "media-1",
				time: 1,
				width: 464,
				height: 260,
			}),
		);
	});

	test("shares keys for times within one quantum", () => {
		const quantum = 1 / 4096;
		const first = buildThumbnailTileKey({
			mediaId: "media-1",
			time: 1,
			width: 232,
			height: 130,
		});
		const second = buildThumbnailTileKey({
			mediaId: "media-1",
			time: 1 + quantum / 4,
			width: 232,
			height: 130,
		});
		expect(second).toBe(first);
	});

	test("splits keys across quantum boundaries", () => {
		const first = buildThumbnailTileKey({
			mediaId: "media-1",
			time: 1,
			width: 232,
			height: 130,
		});
		const second = buildThumbnailTileKey({
			mediaId: "media-1",
			time: 1 + 1 / 4096,
			width: 232,
			height: 130,
		});
		expect(second).not.toBe(first);
	});
});
