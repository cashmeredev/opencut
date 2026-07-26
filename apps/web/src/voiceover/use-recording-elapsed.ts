import { useEffect, useState } from "react";
import { useVoiceOverStore } from "@/voiceover/voiceover-store";

const ELAPSED_TICK_MS = 250;

/**
 * Elapsed seconds of the active voice-over recording, ticking while recording.
 * Returns 0 when idle.
 */
export function useRecordingElapsedSeconds(): number {
	const status = useVoiceOverStore((state) => state.status);
	const startedAtMs = useVoiceOverStore((state) => state.startedAtMs);
	const [elapsedSeconds, setElapsedSeconds] = useState(0);

	useEffect(() => {
		if (status !== "recording" || startedAtMs === null) {
			return;
		}
		const interval = setInterval(() => {
			setElapsedSeconds((Date.now() - startedAtMs) / 1000);
		}, ELAPSED_TICK_MS);
		return () => clearInterval(interval);
	}, [status, startedAtMs]);

	return status === "recording" ? elapsedSeconds : 0;
}
