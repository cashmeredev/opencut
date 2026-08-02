import { describe, expect, test } from "bun:test";
import { registerDefaultEffects } from "@/effects";
import { buildDefaultEffectInstance } from "@/effects";
import { applyMotionEffects, sampleEasingBezier } from "@/effects/motion";
import type { Transform } from "@/rendering";
import { GraphicNode } from "@/services/renderer/nodes/graphic-node";
import { resolveRenderTree } from "@/services/renderer/resolve";
import type { CanvasRenderer } from "@/services/renderer/canvas-renderer";

registerDefaultEffects();

const baseTransform: Transform = {
	scaleX: 1,
	scaleY: 1,
	position: { x: 0, y: 0 },
	rotate: 0,
};

function applyZoomIn({
	localTime,
	duration = 10,
	intensity = 100,
	easing = "linear",
}: {
	localTime: number;
	duration?: number;
	intensity?: number;
	easing?: string;
}) {
	const effect = buildDefaultEffectInstance({ effectType: "zoom-in" });
	effect.params.intensity = intensity;
	effect.params.easing = easing;
	return applyMotionEffects({
		baseTransform,
		effects: [effect],
		animations: undefined,
		localTime,
		duration,
		canvasWidth: 1920,
		canvasHeight: 1080,
	});
}

describe("sampleEasingBezier", () => {
	test("linear easing is identity", () => {
		expect(sampleEasingBezier({ bezier: [0, 0, 1, 1], x: 0.37 })).toBeCloseTo(
			0.37,
			5,
		);
	});

	test("preserves endpoints", () => {
		expect(sampleEasingBezier({ bezier: [0.4, 0, 0.2, 1], x: 0 })).toBeCloseTo(
			0,
			5,
		);
		expect(sampleEasingBezier({ bezier: [0.4, 0, 0.2, 1], x: 1 })).toBeCloseTo(
			1,
			5,
		);
	});

	test("ease-in lags behind linear progress mid-way", () => {
		const eased = sampleEasingBezier({ bezier: [0.8, 0, 1, 1], x: 0.5 });
		expect(eased).toBeLessThan(0.5);
	});
});

describe("applyMotionEffects", () => {
	test("zoom-in ramps scale from 1 to full range over the clip", () => {
		expect(applyZoomIn({ localTime: 0 }).scaleX).toBeCloseTo(1, 5);
		expect(applyZoomIn({ localTime: 10 }).scaleX).toBeCloseTo(1.35, 5);
	});

	test("intensity scales the effect proportionally", () => {
		const full = applyZoomIn({ localTime: 10, intensity: 100 });
		const fifth = applyZoomIn({ localTime: 10, intensity: 20 });
		expect(fifth.scaleX).toBeCloseTo(1.07, 5);
		expect((fifth.scaleX - 1) / (full.scaleX - 1)).toBeCloseTo(0.2, 5);
	});

	test("eased zoom is non-linear at the midpoint", () => {
		const eased = applyZoomIn({ localTime: 5, easing: "ease-in" });
		expect(eased.scaleX).toBeLessThan(1.175);
		expect(eased.scaleX).toBeGreaterThan(1);
	});

	test("zoom-out starts zoomed and settles at 1", () => {
		const effect = buildDefaultEffectInstance({ effectType: "zoom-out" });
		effect.params.easing = "linear";
		const apply = (localTime: number) =>
			applyMotionEffects({
				baseTransform,
				effects: [effect],
				animations: undefined,
				localTime,
				duration: 10,
				canvasWidth: 1920,
				canvasHeight: 1080,
			});
		expect(apply(0).scaleX).toBeCloseTo(1.35, 5);
		expect(apply(10).scaleX).toBeCloseTo(1, 5);
	});

	test("pan-right drifts position across the canvas with zoom headroom", () => {
		const effect = buildDefaultEffectInstance({ effectType: "pan-right" });
		effect.params.easing = "linear";
		const apply = (localTime: number) =>
			applyMotionEffects({
				baseTransform,
				effects: [effect],
				animations: undefined,
				localTime,
				duration: 10,
				canvasWidth: 1920,
				canvasHeight: 1080,
			});
		const start = apply(0);
		const end = apply(10);
		expect(start.position.x).toBeCloseTo(-0.06 * 1920, 3);
		expect(end.position.x).toBeCloseTo(0.06 * 1920, 3);
		expect(end.scaleX).toBeCloseTo(1.15, 5);
		expect(end.scaleX).toBe(end.scaleY);
	});

	test("multiplies onto an existing base transform", () => {
		const scaledBase: Transform = { ...baseTransform, scaleX: 2, scaleY: 2 };
		const effect = buildDefaultEffectInstance({ effectType: "zoom-in" });
		effect.params.easing = "linear";
		const resolved = applyMotionEffects({
			baseTransform: scaledBase,
			effects: [effect],
			animations: undefined,
			localTime: 10,
			duration: 10,
			canvasWidth: 1920,
			canvasHeight: 1080,
		});
		expect(resolved.scaleX).toBeCloseTo(2.7, 5);
	});

	test("ignores disabled effects and effects without motion", () => {
		const motion = buildDefaultEffectInstance({ effectType: "zoom-in" });
		motion.enabled = false;
		const blur = buildDefaultEffectInstance({ effectType: "blur" });
		const resolved = applyMotionEffects({
			baseTransform,
			effects: [motion, blur],
			animations: undefined,
			localTime: 5,
			duration: 10,
			canvasWidth: 1920,
			canvasHeight: 1080,
		});
		expect(resolved).toEqual(baseTransform);
	});
});

describe("renderer wiring", () => {
	test("resolveRenderTree applies motion presets to the resolved transform", async () => {
		const effect = buildDefaultEffectInstance({ effectType: "zoom-in" });
		effect.params.easing = "linear";
		const node = new GraphicNode({
			definitionId: "rectangle",
			params: {},
			duration: 10,
			timeOffset: 0,
			trimStart: 0,
			trimEnd: 0,
			transform: baseTransform,
			opacity: 1,
			effects: [effect],
		});
		const renderer = { width: 1920, height: 1080 } as CanvasRenderer;

		await resolveRenderTree({ node, renderer, time: 0 });
		expect(node.resolved?.transform.scaleX).toBeCloseTo(1, 5);

		await resolveRenderTree({ node, renderer, time: 5 });
		expect(node.resolved?.transform.scaleX).toBeCloseTo(1.175, 5);
	});
});
