"use client";

import { useEffect, useRef, useCallback } from "react";
import { PanelView } from "@/components/editor/panels/assets/views/base-panel";
import { DraggableItem } from "@/components/editor/panels/assets/draggable-item";
import { effectsRegistry, EFFECT_TARGET_ELEMENT_TYPES } from "@/effects";
import { effectPreviewService } from "@/services/renderer/effect-preview";
import { useEditor } from "@/editor/use-editor";
import { buildEffectElement } from "@/timeline/element-utils";
import type { EffectDefinition } from "@/effects/types";

export function EffectsView() {
	const effects = effectsRegistry.getAll();

	return (
		<PanelView title="Effects">
			<EffectsGrid effects={effects} />
		</PanelView>
	);
}

function EffectsGrid({ effects }: { effects: EffectDefinition[] }) {
	return (
		<div
			className="grid gap-2"
			style={{ gridTemplateColumns: "repeat(auto-fill, minmax(96px, 1fr))" }}
		>
			{effects.map((effect) => (
				<EffectItem key={effect.type} effect={effect} />
			))}
		</div>
	);
}

function EffectPreviewCanvas({ effectType }: { effectType: string }) {
	const canvasRef = useRef<HTMLCanvasElement>(null);

	useEffect(() => {
		const render = () => {
			if (canvasRef.current) {
				effectPreviewService.renderPreview({
					effectType,
					params: {},
					targetCanvas: canvasRef.current,
				});
			}
		};

		render();
		return effectPreviewService.onPreviewImageReady({ callback: render });
	}, [effectType]);

	return <canvas ref={canvasRef} className="size-full" />;
}

const MOTION_ARROW_DIRECTIONS: Record<string, [number, number]> = {
	"pan-left": [-1, 0],
	"pan-right": [1, 0],
	"pan-up": [0, -1],
	"pan-down": [0, 1],
};

function drawArrow({
	context,
	fromX,
	fromY,
	toX,
	toY,
}: {
	context: CanvasRenderingContext2D;
	fromX: number;
	fromY: number;
	toX: number;
	toY: number;
}) {
	const headLength = 10;
	const angle = Math.atan2(toY - fromY, toX - fromX);
	context.beginPath();
	context.moveTo(fromX, fromY);
	context.lineTo(toX, toY);
	context.moveTo(toX, toY);
	context.lineTo(
		toX - headLength * Math.cos(angle - Math.PI / 6),
		toY - headLength * Math.sin(angle - Math.PI / 6),
	);
	context.moveTo(toX, toY);
	context.lineTo(
		toX - headLength * Math.cos(angle + Math.PI / 6),
		toY - headLength * Math.sin(angle + Math.PI / 6),
	);
	context.stroke();
}

function MotionPreviewCanvas({ effectType }: { effectType: string }) {
	const canvasRef = useRef<HTMLCanvasElement>(null);

	useEffect(() => {
		const canvas = canvasRef.current;
		const context = canvas?.getContext("2d");
		if (!canvas || !context) return;

		const size = canvas.width;
		const center = size / 2;
		context.clearRect(0, 0, size, size);
		context.fillStyle = "#17171c";
		context.fillRect(0, 0, size, size);
		context.strokeStyle = "rgba(255, 255, 255, 0.35)";
		context.lineWidth = 2;
		context.strokeRect(size * 0.2, size * 0.28, size * 0.6, size * 0.44);
		context.strokeStyle = "#ffffff";
		context.lineWidth = 4;
		context.lineCap = "round";

		const panDirection = MOTION_ARROW_DIRECTIONS[effectType];
		if (panDirection) {
			const [dx, dy] = panDirection;
			drawArrow({
				context,
				fromX: center - dx * size * 0.18,
				fromY: center - dy * size * 0.18,
				toX: center + dx * size * 0.18,
				toY: center + dy * size * 0.18,
			});
			return;
		}

		const inward = effectType === "zoom-out";
		const spread = size * 0.16;
		const reach = size * 0.3;
		for (const [dx, dy] of [
			[1, 1],
			[-1, -1],
		] as const) {
			const innerX = center + dx * (inward ? reach : spread);
			const innerY = center + dy * (inward ? reach : spread) * 0.7;
			const outerX = center + dx * (inward ? spread : reach);
			const outerY = center + dy * (inward ? spread : reach) * 0.7;
			drawArrow({ context, fromX: innerX, fromY: innerY, toX: outerX, toY: outerY });
		}
	}, [effectType]);

	return (
		<canvas
			ref={canvasRef}
			width={effectPreviewService.PREVIEW_SIZE}
			height={effectPreviewService.PREVIEW_SIZE}
			className="size-full"
		/>
	);
}

function EffectItem({ effect }: { effect: EffectDefinition }) {
	const editor = useEditor();

	const handleAddToTimeline = useCallback(() => {
		const currentTime = editor.playback.getCurrentTime();
		const element = buildEffectElement({
			effectType: effect.type,
			startTime: currentTime,
		});

		editor.timeline.insertElement({
			placement: { mode: "auto", trackType: "effect" },
			element,
		});
	}, [editor, effect.type]);

	const preview = effect.motion ? (
		<MotionPreviewCanvas effectType={effect.type} />
	) : (
		<EffectPreviewCanvas effectType={effect.type} />
	);

	return (
		<DraggableItem
			name={effect.name}
			preview={preview}
			dragData={{
				id: effect.type,
				name: effect.name,
				type: "effect",
				effectType: effect.type,
				targetElementTypes: EFFECT_TARGET_ELEMENT_TYPES,
			}}
			onAddToTimeline={
				effect.supportsStandalone === false ? undefined : handleAddToTimeline
			}
			aspectRatio={1}
			isRounded
			variant="card"
			containerClassName="w-full"
		/>
	);
}
