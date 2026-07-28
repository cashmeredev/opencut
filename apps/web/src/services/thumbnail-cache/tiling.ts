export const MEDIA_TILE_ASPECT_RATIO = 16 / 9;
export const FILMSTRIP_MIN_INTERVAL_SECONDS = 0.1;
export const FILMSTRIP_MAX_INTERVAL_SECONDS = 64;
export const FILMSTRIP_OVERSCAN_SLOTS = 2;

const SOURCE_TIME_END_PADDING_SECONDS = 0.05;
const TILE_TIME_QUANTUM_PER_SECOND = 4096;

export function getFilmstripInterval({
	pixelsPerSecond,
	minTileWidthPx,
}: {
	pixelsPerSecond: number;
	minTileWidthPx: number;
}): number {
	if (
		!Number.isFinite(pixelsPerSecond) ||
		pixelsPerSecond <= 0 ||
		!Number.isFinite(minTileWidthPx) ||
		minTileWidthPx <= 0
	) {
		return FILMSTRIP_MAX_INTERVAL_SECONDS;
	}

	const ideal = minTileWidthPx / pixelsPerSecond;
	const clamped = Math.min(
		FILMSTRIP_MAX_INTERVAL_SECONDS,
		Math.max(FILMSTRIP_MIN_INTERVAL_SECONDS, ideal),
	);
	return 2 ** Math.round(Math.log2(clamped));
}

export function getFilmstripSlotCount({
	clipDurationSec,
	interval,
}: {
	clipDurationSec: number;
	interval: number;
}): number {
	if (
		!Number.isFinite(clipDurationSec) ||
		clipDurationSec <= 0 ||
		interval <= 0
	) {
		return 0;
	}
	return Math.max(1, Math.ceil(clipDurationSec / interval - 1e-6));
}

export function getVisibleSlotRange({
	slotCount,
	tileWidthPx,
	visibleLeftPx,
	visibleRightPx,
	overscanSlots = FILMSTRIP_OVERSCAN_SLOTS,
}: {
	slotCount: number;
	tileWidthPx: number;
	visibleLeftPx: number;
	visibleRightPx: number;
	overscanSlots?: number;
}): { firstSlot: number; lastSlot: number } {
	if (slotCount <= 0 || tileWidthPx <= 0 || visibleRightPx <= visibleLeftPx) {
		return { firstSlot: 0, lastSlot: -1 };
	}

	const firstSlot = Math.max(
		0,
		Math.floor(visibleLeftPx / tileWidthPx) - overscanSlots,
	);
	const lastSlot = Math.min(
		slotCount - 1,
		Math.ceil(visibleRightPx / tileWidthPx) - 1 + overscanSlots,
	);
	return { firstSlot, lastSlot };
}

export function getFilmstripSlotSourceTime({
	slot,
	interval,
	trimStartSec,
	rate,
	assetDurationSec,
}: {
	slot: number;
	interval: number;
	trimStartSec: number;
	rate: number;
	assetDurationSec?: number;
}): number {
	const safeRate = Number.isFinite(rate) && rate > 0 ? rate : 1;
	const raw = Math.max(0, trimStartSec + slot * interval * safeRate);
	if (
		assetDurationSec === undefined ||
		!Number.isFinite(assetDurationSec) ||
		assetDurationSec <= 0
	) {
		return raw;
	}
	return Math.min(
		raw,
		Math.max(0, assetDurationSec - SOURCE_TIME_END_PADDING_SECONDS),
	);
}

export function buildThumbnailTileKey({
	mediaId,
	time,
	width,
	height,
}: {
	mediaId: string;
	time: number;
	width: number;
	height: number;
}): string {
	const quantizedTime = Math.round(
		Math.max(0, time) * TILE_TIME_QUANTUM_PER_SECOND,
	);
	return `${mediaId}:${width}x${height}:${quantizedTime}`;
}
