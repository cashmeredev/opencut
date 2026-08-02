import type { ParamDefinition, ParamValues } from "@/params";
import type { EffectDefinition, MotionOffset } from "@/effects/types";
import { MOTION_EASING_OPTIONS, getMotionIntensity } from "@/effects/motion";

const ZOOM_RANGE = 0.35;
const PAN_ZOOM_HEADROOM = 0.15;
const PAN_TRAVEL = 0.06;

function motionParams(): ParamDefinition[] {
	return [
		{
			key: "intensity",
			label: "Intensity",
			type: "number",
			default: 100,
			min: 0,
			max: 100,
			step: 1,
			keyframable: true,
		},
		{
			key: "easing",
			label: "Easing",
			type: "select",
			default: "smooth",
			options: MOTION_EASING_OPTIONS,
		},
	];
}

function zoomOffset({
	effectParams,
	progress,
	direction,
}: {
	effectParams: ParamValues;
	progress: number;
	direction: 1 | -1;
}): MotionOffset {
	const intensity = getMotionIntensity({ effectParams });
	const amount =
		direction === 1 ? progress * intensity : (1 - progress) * intensity;
	return {
		scale: 1 + ZOOM_RANGE * amount,
		translateX: 0,
		translateY: 0,
	};
}

function panOffset({
	effectParams,
	progress,
	axis,
	direction,
}: {
	effectParams: ParamValues;
	progress: number;
	axis: "x" | "y";
	direction: 1 | -1;
}): MotionOffset {
	const intensity = getMotionIntensity({ effectParams });
	const travel = (progress - 0.5) * 2 * PAN_TRAVEL * intensity * direction;
	return {
		scale: 1 + PAN_ZOOM_HEADROOM * intensity,
		translateX: axis === "x" ? travel : 0,
		translateY: axis === "y" ? travel : 0,
	};
}

function buildMotionDefinition({
	type,
	name,
	keywords,
	evaluate,
}: Pick<EffectDefinition, "type" | "name" | "keywords"> & {
	evaluate: NonNullable<EffectDefinition["motion"]>["evaluate"];
}): EffectDefinition {
	return {
		type,
		name,
		keywords,
		params: motionParams(),
		renderer: { passes: [] },
		motion: { evaluate },
		supportsStandalone: false,
	};
}

export const motionEffectDefinitions: EffectDefinition[] = [
	buildMotionDefinition({
		type: "zoom-in",
		name: "Zoom In",
		keywords: ["zoom", "ken burns", "scale", "push in"],
		evaluate: ({ effectParams, progress }) =>
			zoomOffset({ effectParams, progress, direction: 1 }),
	}),
	buildMotionDefinition({
		type: "zoom-out",
		name: "Zoom Out",
		keywords: ["zoom", "ken burns", "scale", "pull out"],
		evaluate: ({ effectParams, progress }) =>
			zoomOffset({ effectParams, progress, direction: -1 }),
	}),
	buildMotionDefinition({
		type: "pan-left",
		name: "Pan Left",
		keywords: ["pan", "ken burns", "slide", "left"],
		evaluate: ({ effectParams, progress }) =>
			panOffset({ effectParams, progress, axis: "x", direction: -1 }),
	}),
	buildMotionDefinition({
		type: "pan-right",
		name: "Pan Right",
		keywords: ["pan", "ken burns", "slide", "right"],
		evaluate: ({ effectParams, progress }) =>
			panOffset({ effectParams, progress, axis: "x", direction: 1 }),
	}),
	buildMotionDefinition({
		type: "pan-up",
		name: "Pan Up",
		keywords: ["pan", "ken burns", "slide", "up", "tilt"],
		evaluate: ({ effectParams, progress }) =>
			panOffset({ effectParams, progress, axis: "y", direction: -1 }),
	}),
	buildMotionDefinition({
		type: "pan-down",
		name: "Pan Down",
		keywords: ["pan", "ken burns", "slide", "down", "tilt"],
		evaluate: ({ effectParams, progress }) =>
			panOffset({ effectParams, progress, axis: "y", direction: 1 }),
	}),
];
