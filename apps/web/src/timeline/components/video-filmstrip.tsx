"use client";

import {
	useCallback,
	useEffect,
	useLayoutEffect,
	useMemo,
	useRef,
	useState,
} from "react";
import { useResizeObserver } from "@/hooks/use-resize-observer";
import type { MediaAsset } from "@/media/types";
import { getEffectiveRateAt } from "@/retime/resolve";
import { thumbnailCache } from "@/services/thumbnail-cache/service";
import {
	getFilmstripInterval,
	getFilmstripSlotCount,
	getFilmstripSlotSourceTime,
	getVisibleSlotRange,
	MEDIA_TILE_ASPECT_RATIO,
} from "@/services/thumbnail-cache/tiling";
import type { VideoElement } from "@/timeline";
import { findScrollParent } from "@/utils/browser";
import { mediaTimeToSeconds } from "@/wasm";

interface VideoFilmstripProps {
	element: VideoElement;
	mediaAsset: MediaAsset;
	pixelsPerSecond: number;
	trackHeight: number;
}

export function VideoFilmstrip({
	element,
	mediaAsset,
	pixelsPerSecond,
	trackHeight,
}: VideoFilmstripProps) {
	const containerRef = useRef<HTMLDivElement>(null);
	const [visibleRange, setVisibleRange] = useState({ left: 0, right: 0 });

	const clipDurationSec = mediaTimeToSeconds({ time: element.duration });
	const trimStartSec = mediaTimeToSeconds({ time: element.trimStart });
	const rate = getEffectiveRateAt({ retime: element.retime });
	const minTileWidthPx = trackHeight * MEDIA_TILE_ASPECT_RATIO;
	const interval = getFilmstripInterval({ pixelsPerSecond, minTileWidthPx });
	const tileWidthPx = interval * pixelsPerSecond;
	const slotCount = getFilmstripSlotCount({ clipDurationSec, interval });

	const updateVisibleRange = useCallback(() => {
		const container = containerRef.current;
		if (!container) {
			return;
		}
		const rect = container.getBoundingClientRect();
		if (rect.width <= 0) {
			return;
		}
		const scrollParent = findScrollParent({ element: container });
		let left = 0;
		let right = rect.width;
		if (scrollParent) {
			const parentRect = scrollParent.getBoundingClientRect();
			left = Math.max(0, parentRect.left - rect.left);
			right = Math.min(rect.width, parentRect.right - rect.left);
		} else {
			left = Math.max(0, -rect.left);
			right = Math.min(rect.width, window.innerWidth - rect.left);
		}
		setVisibleRange((previous) =>
			previous.left === left && previous.right === right
				? previous
				: { left, right },
		);
	}, []);

	useLayoutEffect(() => {
		updateVisibleRange();
	}, [updateVisibleRange, pixelsPerSecond, clipDurationSec, interval]);

	useEffect(() => {
		const container = containerRef.current;
		if (!container) {
			return;
		}
		const scrollParent = findScrollParent({ element: container });
		if (!scrollParent) {
			return;
		}
		scrollParent.addEventListener("scroll", updateVisibleRange, {
			passive: true,
		});
		return () => scrollParent.removeEventListener("scroll", updateVisibleRange);
	}, [updateVisibleRange]);

	useResizeObserver({ ref: containerRef, onResize: updateVisibleRange });

	const { firstSlot, lastSlot } = getVisibleSlotRange({
		slotCount,
		tileWidthPx,
		visibleLeftPx: visibleRange.left,
		visibleRightPx: visibleRange.right,
	});

	const bitmapSize = useMemo(() => {
		const dpr =
			typeof window === "undefined" ? 1 : window.devicePixelRatio || 1;
		return {
			width: Math.max(1, Math.round(minTileWidthPx * dpr)),
			height: Math.max(1, Math.round(trackHeight * dpr)),
		};
	}, [minTileWidthPx, trackHeight]);

	const slots: number[] = [];
	for (let slot = firstSlot; slot <= lastSlot; slot++) {
		slots.push(slot);
	}

	return (
		<div
			ref={containerRef}
			className="pointer-events-none absolute inset-0 overflow-hidden"
		>
			{slots.map((slot) => (
				<FilmstripTile
					key={slot}
					mediaId={mediaAsset.id}
					file={mediaAsset.file}
					sourceTimeSec={getFilmstripSlotSourceTime({
						slot,
						interval,
						trimStartSec,
						rate,
						assetDurationSec: mediaAsset.duration,
					})}
					leftPx={slot * tileWidthPx}
					widthPx={tileWidthPx}
					heightPx={trackHeight}
					bitmapWidth={bitmapSize.width}
					bitmapHeight={bitmapSize.height}
				/>
			))}
		</div>
	);
}

function FilmstripTile({
	mediaId,
	file,
	sourceTimeSec,
	leftPx,
	widthPx,
	heightPx,
	bitmapWidth,
	bitmapHeight,
}: {
	mediaId: string;
	file: File;
	sourceTimeSec: number;
	leftPx: number;
	widthPx: number;
	heightPx: number;
	bitmapWidth: number;
	bitmapHeight: number;
}) {
	const requestKey = `${mediaId}:${sourceTimeSec}:${bitmapWidth}x${bitmapHeight}`;
	const [loaded, setLoaded] = useState<{
		requestKey: string;
		url: string;
	} | null>(null);

	useEffect(() => {
		let isCancelled = false;
		void thumbnailCache
			.getTile({
				mediaId,
				file,
				time: sourceTimeSec,
				width: bitmapWidth,
				height: bitmapHeight,
			})
			.then((url) => {
				if (!isCancelled && url) {
					setLoaded({ requestKey, url });
				}
			});
		return () => {
			isCancelled = true;
		};
	}, [mediaId, file, sourceTimeSec, bitmapWidth, bitmapHeight, requestKey]);

	if (!loaded || loaded.requestKey !== requestKey) {
		return null;
	}

	return (
		<div
			className="absolute top-0"
			style={{
				left: `${leftPx}px`,
				width: `${widthPx}px`,
				height: `${heightPx}px`,
				backgroundImage: `url(${loaded.url})`,
				backgroundSize: "100% 100%",
			}}
		/>
	);
}
