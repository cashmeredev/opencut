/**
 * Voice-over recording UI state.
 * Transient by nature — a recording never survives a reload, so unlike most
 * stores here this one is intentionally NOT persisted (same pattern as
 * editor-store / properties-store).
 */

import { create } from "zustand";
import type { MediaTime } from "@/wasm";

export type VoiceOverStatus = "idle" | "recording";

interface VoiceOverStore {
	status: VoiceOverStatus;
	startedAtMs: number | null;
	playheadStartTime: MediaTime | null;
	startRecording: ({ playheadTime }: { playheadTime: MediaTime }) => void;
	stopRecording: () => void;
}

export const useVoiceOverStore = create<VoiceOverStore>()((set) => ({
	status: "idle",
	startedAtMs: null,
	playheadStartTime: null,

	startRecording: ({ playheadTime }) => {
		set({
			status: "recording",
			startedAtMs: Date.now(),
			playheadStartTime: playheadTime,
		});
	},

	stopRecording: () => {
		set({
			status: "idle",
			startedAtMs: null,
			playheadStartTime: null,
		});
	},
}));
