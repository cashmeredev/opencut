import type { ElementAnimations } from "@/animation/types";
import type { NormalizedCubicBezier } from "@/animation/types";
import { getBezierPoint } from "@/animation/bezier";
import { resolveEffectParamsAtTime } from "@/animation/effect-param-channel";
import type { ParamValues } from "@/params";
import { clamp } from "@/utils/math";
import type { Transform } from "@/rendering";
import type { Effect } from "@/effects/types";
import { effectsRegistry } from "@/effects/registry";
import { BUILTIN_PRESETS } from "@/timeline/components/graph-editor/easing-presets";

const BEZIER_SOLVE_ITERATIONS = 20;

export const MOTION_EASING_OPTIONS = BUILTIN_PRESETS.map((preset) => ({
	value: preset.id,
	label: preset.label,
}));

export function getMotionIntensity({
	effectParams,
}: {
	effectParams: ParamValues;
}): number {
	const raw = effectParams.intensity;
	const intensity = typeof raw === "number" ? raw : Number.parseFloat(String(raw));
	if (!Number.isFinite(intensity)) return 1;
	return clamp({ value: intensity / 100, min: 0, max: 1 });
}

export function sampleEasingBezier({
	bezier,
	x,
}: {
	bezier: NormalizedCubicBezier;
	x: number;
}): number {
	const [x1, y1, x2, y2] = bezier;
	let lower = 0;
	let upper = 1;
	for (let iteration = 0; iteration < BEZIER_SOLVE_ITERATIONS; iteration++) {
		const mid = (lower + upper) / 2;
		const estimate = getBezierPoint({
			progress: mid,
			p0: 0,
			p1: x1,
			p2: x2,
			p3: 1,
		});
		if (estimate < x) {
			lower = mid;
		} else {
			upper = mid;
		}
	}
	return getBezierPoint({
		progress: (lower + upper) / 2,
		p0: 0,
		p1: y1,
		p2: y2,
		p3: 1,
	});
}

export function applyMotionEffects({
	baseTransform,
	effects,
	animations,
	localTime,
	duration,
	canvasWidth,
	canvasHeight,
}: {
	baseTransform: Transform;
	effects: Effect[] | undefined;
	animations: ElementAnimations | undefined;
	localTime: number;
	duration: number;
	canvasWidth: number;
	canvasHeight: number;
}): Transform {
	const motionEffects = (effects ?? []).filter(
		(effect) => effect.enabled && effectsRegistry.get(effect.type).motion,
	);
	if (motionEffects.length === 0 || duration <= 0) {
		return baseTransform;
	}

	const linearProgress = clamp({
		value: localTime / duration,
		min: 0,
		max: 1,
	});

	let scale = 1;
	let translateX = 0;
	let translateY = 0;

	for (const effect of motionEffects) {
		const definition = effectsRegistry.get(effect.type);
		if (!definition.motion) continue;
		const resolvedParams = resolveEffectParamsAtTime({
			effectId: effect.id,
			params: effect.params,
			animations,
			localTime,
		});
		const progress = sampleEasingBezier({
			bezier:
				BUILTIN_PRESETS.find((entry) => entry.id === resolvedParams.easing)
					?.value ?? [0.25, 0.1, 0.25, 1],
			x: linearProgress,
		});
		const offset = definition.motion.evaluate({
			effectParams: resolvedParams,
			progress,
		});
		scale *= offset.scale;
		translateX += offset.translateX;
		translateY += offset.translateY;
	}

	return {
		scaleX: baseTransform.scaleX * scale,
		scaleY: baseTransform.scaleY * scale,
		position: {
			x: baseTransform.position.x + translateX * canvasWidth,
			y: baseTransform.position.y + translateY * canvasHeight,
		},
		rotate: baseTransform.rotate,
	};
}
