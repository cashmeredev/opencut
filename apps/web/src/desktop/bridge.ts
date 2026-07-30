export type DesktopExportBegin = {
	width: number;
	height: number;
	fpsNumerator: number;
	fpsDenominator: number;
	format: "mp4" | "webm";
	quality: "low" | "medium" | "high" | "very_high";
	hasAudio: boolean;
	sampleRate: number;
	channels: number;
	defaultFileName: string;
};

export type OpenCutDesktop = {
	isDesktop: true;
	exportBegin: (
		opts: DesktopExportBegin,
	) => Promise<{ id: string; filePath: string } | null>;
	exportWriteAudio: (id: string, pcm: ArrayBuffer) => Promise<void>;
	exportWriteFrame: (id: string, rgba: ArrayBuffer) => Promise<void>;
	exportFinish: (id: string) => Promise<void>;
	exportCancel: (id: string) => Promise<void>;
	revealFile: (path: string) => Promise<void>;
};

declare global {
	interface Window {
		opencutDesktop?: OpenCutDesktop;
	}
}

export function getDesktopBridge(): OpenCutDesktop | null {
	if (typeof window === "undefined") return null;
	return window.opencutDesktop ?? null;
}
