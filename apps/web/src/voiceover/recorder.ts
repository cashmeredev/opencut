/**
 * Voice-over recording via MediaRecorder.
 * Pure helpers (mime type picking, file extension, elapsed formatting, error
 * messages) are exported for unit tests; the recorder singleton below owns the
 * actual microphone stream.
 */

export const PREFERRED_MIME_TYPES = [
	"audio/webm;codecs=opus",
	"audio/webm",
	"audio/ogg;codecs=opus",
	"audio/ogg",
	"audio/mp4",
] as const;

export function pickSupportedMimeType({
	isTypeSupported,
	candidates = PREFERRED_MIME_TYPES,
}: {
	isTypeSupported: (mimeType: string) => boolean;
	candidates?: readonly string[];
}): string | null {
	for (const mimeType of candidates) {
		if (isTypeSupported(mimeType)) {
			return mimeType;
		}
	}
	return null;
}

const EXTENSION_BY_MIME_SUBTYPE: Record<string, string> = {
	webm: "webm",
	ogg: "ogg",
	mp4: "m4a",
	mpeg: "mp3",
	wav: "wav",
};

export function getFileExtensionForMimeType({
	mimeType,
}: {
	mimeType: string;
}): string {
	const subtype = mimeType.split("/").at(1)?.split(";").at(0)?.trim() ?? "";
	return EXTENSION_BY_MIME_SUBTYPE[subtype] ?? "audio";
}

export function formatElapsedSeconds({
	seconds,
}: {
	seconds: number;
}): string {
	const totalSeconds = Math.max(0, Math.floor(seconds));
	const minutes = Math.floor(totalSeconds / 60);
	const remainder = totalSeconds % 60;
	return `${minutes}:${String(remainder).padStart(2, "0")}`;
}

export function getVoiceOverErrorMessage({
	error,
}: {
	error: unknown;
}): string {
	if (error instanceof DOMException) {
		switch (error.name) {
			case "NotAllowedError":
			case "SecurityError":
				return "Microphone access denied. Allow microphone access in your browser settings to record a voice-over.";
			case "NotFoundError":
			case "OverconstrainedError":
				return "No microphone found. Connect a microphone and try again.";
			case "NotReadableError":
				return "The microphone is already in use by another application.";
			default:
				return "Could not start voice-over recording.";
		}
	}
	if (error instanceof Error && error.message) {
		return error.message;
	}
	return "Could not start voice-over recording.";
}

export interface VoiceOverRecordingResult {
	blob: Blob;
	mimeType: string;
	durationSeconds: number;
}

interface ActiveRecording {
	recorder: MediaRecorder;
	stream: MediaStream;
	chunks: Blob[];
	mimeType: string;
	startedAtMs: number;
}

let activeRecording: ActiveRecording | null = null;

function stopStreamTracks({ stream }: { stream: MediaStream }): void {
	for (const track of stream.getTracks()) {
		track.stop();
	}
}

export async function startVoiceOverRecording(): Promise<void> {
	if (activeRecording) {
		throw new Error("A voice-over recording is already in progress");
	}

	const mediaDevices = navigator.mediaDevices;
	if (!mediaDevices?.getUserMedia) {
		throw new Error("Voice-over recording is not supported in this browser");
	}

	const stream = await mediaDevices.getUserMedia({ audio: true });
	try {
		const mimeType = pickSupportedMimeType({
			isTypeSupported: (candidate) => MediaRecorder.isTypeSupported(candidate),
		});
		const recorder = mimeType
			? new MediaRecorder(stream, { mimeType })
			: new MediaRecorder(stream);
		const chunks: Blob[] = [];
		recorder.addEventListener("dataavailable", (event) => {
			if (event.data.size > 0) {
				chunks.push(event.data);
			}
		});
		recorder.start();
		activeRecording = {
			recorder,
			stream,
			chunks,
			mimeType: recorder.mimeType || mimeType || "audio/webm",
			startedAtMs: performance.now(),
		};
	} catch (error) {
		stopStreamTracks({ stream });
		throw error;
	}
}

export function stopVoiceOverRecording(): Promise<VoiceOverRecordingResult> {
	const recording = activeRecording;
	if (!recording) {
		return Promise.reject(
			new Error("No voice-over recording is in progress"),
		);
	}
	activeRecording = null;

	const { promise, resolve, reject } =
		Promise.withResolvers<VoiceOverRecordingResult>();
	recording.recorder.addEventListener("stop", () => {
		stopStreamTracks({ stream: recording.stream });
		resolve({
			blob: new Blob(recording.chunks, { type: recording.mimeType }),
			mimeType: recording.mimeType,
			durationSeconds: Math.max(
				0,
				(performance.now() - recording.startedAtMs) / 1000,
			),
		});
	});
	recording.recorder.addEventListener("error", () => {
		stopStreamTracks({ stream: recording.stream });
		reject(new Error("Voice-over recording failed"));
	});
	recording.recorder.stop();
	return promise;
}

export function cancelVoiceOverRecording(): void {
	const recording = activeRecording;
	if (!recording) {
		return;
	}
	activeRecording = null;
	try {
		recording.recorder.stop();
	} catch {
		// recorder may already be stopped; tracks are released either way
	}
	stopStreamTracks({ stream: recording.stream });
}
