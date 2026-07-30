import { app, BrowserWindow, dialog, ipcMain, shell } from "electron";
import { type ChildProcessByStdio, spawn } from "node:child_process";
import type { Readable, Writable } from "node:stream";
import { randomUUID } from "node:crypto";
import { unlink, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import ffmpegStaticPath from "ffmpeg-static";

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

type ExportQuality = DesktopExportBegin["quality"];

type FfmpegProcess = ChildProcessByStdio<Writable, null, Readable>;

type ExportSession = {
	id: string;
	filePath: string;
	opts: DesktopExportBegin;
	audioPath: string | null;
	process: FfmpegProcess | null;
	exitCode: Promise<number> | null;
	stderrTail: string;
};

const MP4_CRF: Record<ExportQuality, number> = {
	low: 28,
	medium: 23,
	high: 19,
	very_high: 15,
};

const WEBM_CRF: Record<ExportQuality, number> = {
	low: 40,
	medium: 32,
	high: 24,
	very_high: 16,
};

const STDERR_TAIL_LIMIT = 8192;

const sessions = new Map<string, ExportSession>();

function resolveFfmpegPath(): string {
	if (!ffmpegStaticPath) {
		throw new Error("ffmpeg binary is not available for this platform");
	}
	if (app.isPackaged) {
		return ffmpegStaticPath.replace("app.asar", "app.asar.unpacked");
	}
	return ffmpegStaticPath;
}

function buildFfmpegArgs(session: ExportSession): string[] {
	const { opts, filePath, audioPath } = session;

	const args = [
		"-y",
		"-f",
		"rawvideo",
		"-pix_fmt",
		"rgba",
		"-s",
		`${opts.width}x${opts.height}`,
		"-r",
		`${opts.fpsNumerator}/${opts.fpsDenominator}`,
		"-i",
		"pipe:0",
	];

	if (audioPath) {
		args.push(
			"-f",
			"f32le",
			"-ar",
			String(opts.sampleRate),
			"-ac",
			String(opts.channels),
			"-i",
			audioPath,
		);
	}

	if (opts.format === "mp4") {
		args.push(
			"-c:v",
			"libx264",
			"-preset",
			"medium",
			"-crf",
			String(MP4_CRF[opts.quality]),
			"-pix_fmt",
			"yuv420p",
			"-color_primaries",
			"bt709",
			"-color_trc",
			"bt709",
			"-colorspace",
			"bt709",
		);
		if (audioPath) {
			args.push("-c:a", "aac", "-b:a", "192k");
		}
		args.push("-movflags", "+faststart");
	} else {
		args.push(
			"-c:v",
			"libvpx-vp9",
			"-deadline",
			"good",
			"-cpu-used",
			"2",
			"-row-mt",
			"1",
			"-crf",
			String(WEBM_CRF[opts.quality]),
			"-b:v",
			"0",
			"-pix_fmt",
			"yuv420p",
		);
		if (audioPath) {
			args.push("-c:a", "libopus", "-b:a", "192k");
		}
	}

	if (audioPath) {
		args.push("-shortest");
	}

	args.push(filePath);
	return args;
}

function getSession(id: string): ExportSession {
	const session = sessions.get(id);
	if (!session) {
		throw new Error(`Unknown export session: ${id}`);
	}
	return session;
}

function toBuffer(data: ArrayBuffer | Uint8Array): Buffer {
	if (data instanceof Uint8Array) {
		return Buffer.from(data.buffer, data.byteOffset, data.byteLength);
	}
	return Buffer.from(data);
}

function spawnFfmpeg(session: ExportSession): FfmpegProcess {
	const proc = spawn(resolveFfmpegPath(), buildFfmpegArgs(session), {
		stdio: ["pipe", "ignore", "pipe"],
	});

	proc.stderr.on("data", (chunk: Buffer) => {
		session.stderrTail = (session.stderrTail + chunk.toString()).slice(
			-STDERR_TAIL_LIMIT,
		);
	});

	session.exitCode = new Promise<number>((resolve, reject) => {
		proc.on("error", reject);
		proc.on("close", (code) => resolve(code ?? 1));
	});
	session.exitCode.catch(() => {});
	session.process = proc;
	return proc;
}

async function teardownSession(session: ExportSession): Promise<void> {
	if (session.process) {
		session.process.kill("SIGKILL");
		await session.exitCode?.catch(() => {});
	}
	if (session.audioPath) {
		await unlink(session.audioPath).catch(() => {});
	}
	await unlink(session.filePath).catch(() => {});
}

export function registerExportHandlers(): void {
	ipcMain.handle("export:begin", async (event, opts: DesktopExportBegin) => {
		const win = BrowserWindow.fromWebContents(event.sender);
		const filters =
			opts.format === "mp4"
				? [{ name: "MP4 Video", extensions: ["mp4"] }]
				: [{ name: "WebM Video", extensions: ["webm"] }];
		const saveOptions = { defaultPath: opts.defaultFileName, filters };
		const result = win
			? await dialog.showSaveDialog(win, saveOptions)
			: await dialog.showSaveDialog(saveOptions);

		if (result.canceled || !result.filePath) {
			return null;
		}

		const id = randomUUID();
		sessions.set(id, {
			id,
			filePath: result.filePath,
			opts,
			audioPath: null,
			process: null,
			exitCode: null,
			stderrTail: "",
		});
		return { id, filePath: result.filePath };
	});

	ipcMain.handle(
		"export:writeAudio",
		async (_event, id: string, pcm: ArrayBuffer | Uint8Array) => {
			const session = getSession(id);
			const audioPath = path.join(os.tmpdir(), `opencut-audio-${id}.f32`);
			await writeFile(audioPath, toBuffer(pcm));
			session.audioPath = audioPath;
		},
	);

	ipcMain.handle(
		"export:writeFrame",
		async (_event, id: string, rgba: ArrayBuffer | Uint8Array) => {
			const session = getSession(id);
			const proc = session.process ?? spawnFfmpeg(session);
			const buffer = toBuffer(rgba);

			await new Promise<void>((resolve, reject) => {
				const onError = (error: Error) => reject(error);
				proc.stdin.once("error", onError);
				if (proc.stdin.write(buffer)) {
					proc.stdin.removeListener("error", onError);
					resolve();
					return;
				}
				proc.stdin.once("drain", () => {
					proc.stdin.removeListener("error", onError);
					resolve();
				});
			});
		},
	);

	ipcMain.handle("export:finish", async (_event, id: string) => {
		const session = getSession(id);
		sessions.delete(id);

		try {
			if (session.process && session.exitCode) {
				session.process.stdin.end();
				const code = await session.exitCode;
				if (code !== 0) {
					const tail = session.stderrTail.trim();
					throw new Error(
						`ffmpeg exited with code ${code}${tail ? `: ${tail}` : ""}`,
					);
				}
			}
		} finally {
			if (session.audioPath) {
				await unlink(session.audioPath).catch(() => {});
			}
		}
	});

	ipcMain.handle("export:cancel", async (_event, id: string) => {
		const session = sessions.get(id);
		if (!session) return;
		sessions.delete(id);
		await teardownSession(session);
	});

	ipcMain.handle("reveal-file", (_event, filePath: string) => {
		shell.showItemInFolder(filePath);
	});
}

export function cancelAllExports(): void {
	for (const session of sessions.values()) {
		session.process?.kill("SIGKILL");
	}
	sessions.clear();
}
