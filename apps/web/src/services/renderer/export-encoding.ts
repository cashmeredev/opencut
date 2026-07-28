import type { ExportQuality } from "@/export";

const videoBitsPerPixelFrame: Record<ExportQuality, number> = {
	low: 0.05,
	medium: 0.1,
	high: 0.2,
	very_high: 0.4,
};

export function computeVideoBitrate({
	width,
	height,
	fps,
	quality,
}: {
	width: number;
	height: number;
	fps: number;
	quality: ExportQuality;
}): number {
	return Math.round(width * height * fps * videoBitsPerPixelFrame[quality]);
}
