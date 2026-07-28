"use client";

import {
	ALL_FORMATS,
	BlobSource,
	CanvasSink,
	Input,
	type WrappedCanvas,
} from "mediabunny";
import { buildThumbnailTileKey } from "./tiling";

const MAX_TILE_ENTRIES = 400;
const MAX_SOURCE_ENTRIES = 8;
const MAX_CONCURRENT_FLUSHES = 2;
const TILE_JPEG_QUALITY = 0.7;
const CANVAS_POOL_SIZE = 2;

interface PendingTile {
	time: number;
	resolve: (url: string | null) => void;
}

interface TileSource {
	width: number;
	height: number;
	init: Promise<CanvasSink | null>;
	pending: PendingTile[];
	flushing: boolean;
	flushScheduled: boolean;
	dispose: () => void;
}

function renderTileDataUrl({
	wrapped,
}: {
	wrapped: WrappedCanvas;
}): string | null {
	const canvas = wrapped.canvas;
	if (canvas instanceof HTMLCanvasElement) {
		return canvas.toDataURL("image/jpeg", TILE_JPEG_QUALITY);
	}
	return null;
}

export class ThumbnailCache {
	private tiles = new Map<string, Promise<string | null>>();
	private sources = new Map<string, TileSource>();
	private activeFlushes = 0;
	private flushQueue: Array<() => void> = [];

	getTile({
		mediaId,
		file,
		time,
		width,
		height,
	}: {
		mediaId: string;
		file: File;
		time: number;
		width: number;
		height: number;
	}): Promise<string | null> {
		const key = buildThumbnailTileKey({ mediaId, time, width, height });
		const existing = this.tiles.get(key);
		if (existing) {
			this.tiles.delete(key);
			this.tiles.set(key, existing);
			return existing;
		}

		const source = this.ensureSource({ mediaId, file, width, height });
		const requested = new Promise<string | null>((resolve) => {
			source.pending.push({ time, resolve });
		});
		this.scheduleFlush({ mediaId });

		const tracked = requested.then((url) => {
			if (url === null && this.tiles.get(key) === tracked) {
				this.tiles.delete(key);
			}
			return url;
		});

		this.tiles.set(key, tracked);
		this.evictTiles();
		return tracked;
	}

	clearMedia({ mediaId }: { mediaId: string }): void {
		for (const key of [...this.tiles.keys()]) {
			if (key.startsWith(`${mediaId}:`)) {
				this.tiles.delete(key);
			}
		}
		const source = this.sources.get(mediaId);
		if (source) {
			this.sources.delete(mediaId);
			this.releaseSource({ source });
		}
	}

	clearAll(): void {
		this.tiles.clear();
		for (const source of this.sources.values()) {
			this.releaseSource({ source });
		}
		this.sources.clear();
	}

	private releaseSource({ source }: { source: TileSource }): void {
		const pending = source.pending.splice(0);
		for (const tile of pending) {
			tile.resolve(null);
		}
		source.dispose();
	}

	private ensureSource({
		mediaId,
		file,
		width,
		height,
	}: {
		mediaId: string;
		file: File;
		width: number;
		height: number;
	}): TileSource {
		const existing = this.sources.get(mediaId);
		if (existing && existing.width === width && existing.height === height) {
			this.sources.delete(mediaId);
			this.sources.set(mediaId, existing);
			return existing;
		}
		if (existing) {
			this.sources.delete(mediaId);
			this.releaseSource({ source: existing });
		}

		const input = new Input({
			source: new BlobSource(file),
			formats: ALL_FORMATS,
		});
		const init = input
			.getPrimaryVideoTrack()
			.then((track) => {
				if (!track) {
					return null;
				}
				return new CanvasSink(track, {
					width,
					height,
					fit: "cover",
					poolSize: CANVAS_POOL_SIZE,
				});
			})
			.catch(() => null);

		const source: TileSource = {
			width,
			height,
			init,
			pending: [],
			flushing: false,
			flushScheduled: false,
			dispose: () => {
				void init.finally(() => input.dispose());
			},
		};
		this.sources.set(mediaId, source);
		this.evictSources();
		return source;
	}

	private evictTiles(): void {
		while (this.tiles.size > MAX_TILE_ENTRIES) {
			const oldestKey = this.tiles.keys().next().value;
			if (oldestKey === undefined) {
				return;
			}
			this.tiles.delete(oldestKey);
		}
	}

	private evictSources(): void {
		while (this.sources.size > MAX_SOURCE_ENTRIES) {
			const oldestKey = this.sources.keys().next().value;
			if (oldestKey === undefined) {
				return;
			}
			const oldest = this.sources.get(oldestKey);
			this.sources.delete(oldestKey);
			if (oldest) {
				this.releaseSource({ source: oldest });
			}
		}
	}

	private scheduleFlush({ mediaId }: { mediaId: string }): void {
		const source = this.sources.get(mediaId);
		if (!source || source.flushScheduled) {
			return;
		}
		source.flushScheduled = true;
		queueMicrotask(() => {
			source.flushScheduled = false;
			void this.flush({ mediaId });
		});
	}

	private async flush({ mediaId }: { mediaId: string }): Promise<void> {
		const source = this.sources.get(mediaId);
		if (!source || source.flushing || source.pending.length === 0) {
			return;
		}
		source.flushing = true;
		const batch = source.pending.splice(0).sort((a, b) => a.time - b.time);

		await this.acquireFlushSlot();
		try {
			const sink = await source.init;
			if (!sink) {
				for (const tile of batch) {
					tile.resolve(null);
				}
				return;
			}

			const iterator = sink.canvasesAtTimestamps(
				batch.map((tile) => tile.time),
			);
			for (const tile of batch) {
				const { value, done } = await iterator.next();
				tile.resolve(
					!done && value ? renderTileDataUrl({ wrapped: value }) : null,
				);
			}
		} catch {
			for (const tile of batch) {
				tile.resolve(null);
			}
		} finally {
			this.releaseFlushSlot();
			source.flushing = false;
			if (source.pending.length > 0) {
				this.scheduleFlush({ mediaId });
			}
		}
	}

	private async acquireFlushSlot(): Promise<void> {
		if (this.activeFlushes < MAX_CONCURRENT_FLUSHES) {
			this.activeFlushes++;
			return;
		}
		await new Promise<void>((resolve) => {
			this.flushQueue.push(resolve);
		});
		this.activeFlushes++;
	}

	private releaseFlushSlot(): void {
		this.activeFlushes--;
		const next = this.flushQueue.shift();
		if (next) {
			next();
		}
	}
}

export const thumbnailCache = new ThumbnailCache();
