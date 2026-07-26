/**
 * Test mock for the `opencut-wasm` package.
 *
 * The published package is wasm-pack bundler-target glue that imports
 * `opencut_wasm_bg.wasm` as an ESM module with instance exports. Bun (1.3.x)
 * does not instantiate `.wasm` imports that way, so any test transitively
 * importing `@/wasm` fails at module evaluation. This file is registered as a
 * bun test preload (root `bunfig.toml` → `[test] preload`) so a faithful
 * pure-TS stand-in for the `opencut-wasm` functions used by
 * `@/wasm/media-time.ts` is in place before any test module evaluates.
 *
 * Semantics mirror rust/crates/time/src/media_time.rs (TICKS_PER_SECOND =
 * 120_000, i64 tick lattice, round-half-away-from-zero).
 */
import { mock } from "bun:test";

const TICKS_PER_SECOND = 120_000;

function ticksPerFrame({
	rate,
}: {
	rate: { numerator: number; denominator: number };
}): number | null {
	if (rate.numerator <= 0 || rate.denominator <= 0) {
		return null;
	}
	return Math.trunc((TICKS_PER_SECOND * rate.denominator) / rate.numerator);
}

function toFrameRound({
	time,
	rate,
}: {
	time: number;
	rate: { numerator: number; denominator: number };
}): number | null {
	const perFrame = ticksPerFrame({ rate });
	if (perFrame === null || perFrame === 0) {
		return null;
	}
	const remainder = ((time % perFrame) + perFrame) % perFrame;
	const floor = Math.floor(time / perFrame);
	return remainder * 2 >= perFrame ? floor + 1 : floor;
}

function fromFrame({
	frame,
	rate,
}: {
	frame: number;
	rate: { numerator: number; denominator: number };
}): number | null {
	const perFrame = ticksPerFrame({ rate });
	if (perFrame === null) {
		return null;
	}
	return frame * perFrame;
}

function floorToFrame({
	time,
	rate,
}: {
	time: number;
	rate: { numerator: number; denominator: number };
}): number | null {
	const perFrame = ticksPerFrame({ rate });
	if (perFrame === null || perFrame === 0) {
		return null;
	}
	return Math.floor(time / perFrame) * perFrame;
}

mock.module("opencut-wasm", () => ({
	TICKS_PER_SECOND: () => TICKS_PER_SECOND,
	mediaTimeFromSeconds: ({ seconds }: { seconds: number }): number | null => {
		if (!Number.isFinite(seconds)) {
			return null;
		}
		// Rust f64::round is half away from zero; Math.round is half up.
		const ticks = seconds * TICKS_PER_SECOND;
		return Math.sign(ticks) * Math.round(Math.abs(ticks));
	},
	mediaTimeToSeconds: ({ time }: { time: number }): number =>
		time / TICKS_PER_SECOND,
	roundToFrame: (options: {
		time: number;
		rate: { numerator: number; denominator: number };
	}): number | null => {
		const frame = toFrameRound(options);
		return frame === null ? null : fromFrame({ frame, rate: options.rate });
	},
	lastFrameTime: (options: {
		duration: number;
		rate: { numerator: number; denominator: number };
	}): number | null => {
		if (options.duration <= 0) {
			return 0;
		}
		return floorToFrame({ time: options.duration - 1, rate: options.rate });
	},
	snappedSeekTime: (options: {
		time: number;
		duration: number;
		rate: { numerator: number; denominator: number };
	}): number | null => {
		const frame = toFrameRound({ time: options.time, rate: options.rate });
		if (frame === null) {
			return null;
		}
		const snapped = fromFrame({ frame, rate: options.rate });
		if (snapped === null) {
			return null;
		}
		return Math.min(Math.max(snapped, 0), options.duration);
	},
	// Timecode parsing is not exercised by command tests; fail loudly rather
	// than silently returning a plausible-but-wrong value.
	parseTimecode: (): null => {
		throw new Error("parseTimecode is not implemented in the test mock");
	},
	// GPU/compositor functions are statically linked through `@/core` →
	// renderer-manager but never called by command tests.
	formatTimecode: (): string => {
		throw new Error("formatTimecode is not implemented in the test mock");
	},
	initializeGpu: (): Promise<void> => {
		throw new Error("initializeGpu is not implemented in the test mock");
	},
	applyEffectPasses: (): never => {
		throw new Error("applyEffectPasses is not implemented in the test mock");
	},
	applyMaskFeather: (): never => {
		throw new Error("applyMaskFeather is not implemented in the test mock");
	},
	initCompositor: (): never => {
		throw new Error("initCompositor is not implemented in the test mock");
	},
	getCompositorCanvas: (): never => {
		throw new Error(
			"getCompositorCanvas is not implemented in the test mock",
		);
	},
	getLastFrameProfile: (): never => {
		throw new Error(
			"getLastFrameProfile is not implemented in the test mock",
		);
	},
	releaseTexture: (): never => {
		throw new Error("releaseTexture is not implemented in the test mock");
	},
	renderFrame: (): never => {
		throw new Error("renderFrame is not implemented in the test mock");
	},
	resizeCompositor: (): never => {
		throw new Error("resizeCompositor is not implemented in the test mock");
	},
	uploadTexture: (): never => {
		throw new Error("uploadTexture is not implemented in the test mock");
	},
}));
