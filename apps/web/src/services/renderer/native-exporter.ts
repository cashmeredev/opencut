import EventEmitter from "eventemitter3";

import type { FrameRate } from "opencut-wasm";
import { TICKS_PER_SECOND } from "@/wasm";
import { frameRateToFloat } from "@/fps/utils";
import { getDesktopBridge } from "@/desktop/bridge";
import type { ExportFormat, ExportQuality } from "@/export";
import { CanvasRenderer } from "./canvas-renderer";
import type { RootNode } from "./nodes/root-node";

type NativeExportParams = {
	width: number;
	height: number;
	fps: FrameRate;
	format: ExportFormat;
	quality: ExportQuality;
	shouldIncludeAudio?: boolean;
	audioBuffer?: AudioBuffer;
	fileName: string;
};

export type NativeExporterEvents = {
	progress: [progress: number];
	complete: [];
	error: [error: Error];
	cancelled: [];
};

export class NativeExporter extends EventEmitter<NativeExporterEvents> {
	private renderer: CanvasRenderer;
	private format: ExportFormat;
	private quality: ExportQuality;
	private shouldIncludeAudio: boolean;
	private audioBuffer?: AudioBuffer;
	private fileName: string;

	private isCancelled = false;

	constructor({
		width,
		height,
		fps,
		format,
		quality,
		shouldIncludeAudio,
		audioBuffer,
		fileName,
	}: NativeExportParams) {
		super();
		this.renderer = new CanvasRenderer({
			width,
			height,
			fps,
		});

		this.format = format;
		this.quality = quality;
		this.shouldIncludeAudio = shouldIncludeAudio ?? false;
		this.audioBuffer = audioBuffer;
		this.fileName = fileName;
	}

	cancel(): void {
		this.isCancelled = true;
	}

	private buildAudioPcm(): Float32Array | null {
		if (!this.shouldIncludeAudio || !this.audioBuffer) return null;

		const { numberOfChannels, length } = this.audioBuffer;
		const pcm = new Float32Array(length * numberOfChannels);
		for (let channel = 0; channel < numberOfChannels; channel++) {
			const channelData = this.audioBuffer.getChannelData(channel);
			for (let i = 0; i < length; i++) {
				pcm[i * numberOfChannels + channel] = channelData[i];
			}
		}
		return pcm;
	}

	async export({ rootNode }: { rootNode: RootNode }): Promise<string | null> {
		const bridge = getDesktopBridge();
		if (!bridge) {
			throw new Error("Desktop bridge is not available");
		}

		const fps = this.renderer.fps;
		const fpsFloat = frameRateToFloat(fps);
		const ticksPerFrame = Math.round(
			(TICKS_PER_SECOND * fps.denominator) / fps.numerator,
		);
		const frameCount = Math.floor(rootNode.duration / ticksPerFrame);

		const pcm = this.buildAudioPcm();

		const begin = await bridge.exportBegin({
			width: this.renderer.width,
			height: this.renderer.height,
			fpsNumerator: fps.numerator,
			fpsDenominator: fps.denominator,
			format: this.format,
			quality: this.quality,
			hasAudio: pcm !== null,
			sampleRate: this.audioBuffer?.sampleRate ?? 48000,
			channels: this.audioBuffer?.numberOfChannels ?? 2,
			defaultFileName: this.fileName,
		});

		if (!begin) {
			this.emit("cancelled");
			return null;
		}

		const { id, filePath } = begin;

		try {
			if (pcm) {
				await bridge.exportWriteAudio(id, pcm.buffer as ArrayBuffer);
			}

			for (let i = 0; i < frameCount; i++) {
				if (this.isCancelled) {
					await bridge.exportCancel(id);
					this.emit("cancelled");
					return null;
				}

				const timeTicks = i * ticksPerFrame;
				await this.renderer.render({ node: rootNode, time: timeTicks });

				const frame = new VideoFrame(this.renderer.getOutputCanvas(), {
					timestamp: Math.round((i * 1e6) / fpsFloat),
					duration: Math.round(1e6 / fpsFloat),
				});
				const size = frame.allocationSize({ format: "RGBA" });
				const buffer = new ArrayBuffer(size);
				await frame.copyTo(buffer, { format: "RGBA" });
				frame.close();

				await bridge.exportWriteFrame(id, buffer);
				this.emit("progress", i / frameCount);
			}

			if (this.isCancelled) {
				await bridge.exportCancel(id);
				this.emit("cancelled");
				return null;
			}

			await bridge.exportFinish(id);
			this.emit("progress", 1);
			this.emit("complete");
			return filePath;
		} catch (error) {
			await bridge.exportCancel(id).catch(() => {});
			throw error;
		}
	}
}
