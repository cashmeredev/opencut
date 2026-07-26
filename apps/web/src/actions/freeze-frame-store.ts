"use client";

import { create } from "zustand";
import { persist } from "zustand/middleware";

export const DEFAULT_FREEZE_DURATION_SECONDS = 3;
export const MAX_FREEZE_DURATION_SECONDS = 600;

export function isValidFreezeDuration({
	seconds,
}: {
	seconds: number;
}): boolean {
	return (
		Number.isFinite(seconds) &&
		seconds > 0 &&
		seconds <= MAX_FREEZE_DURATION_SECONDS
	);
}

type DurationResolver = ({ seconds }: { seconds: number | null }) => void;

interface FreezeFrameState {
	isOpen: boolean;
	lastDurationSeconds: number;
	resolver: DurationResolver | null;
	requestDuration: () => Promise<number | null>;
	confirmDuration: ({ seconds }: { seconds: number }) => void;
	cancelDuration: () => void;
}

export const useFreezeFrameStore = create<FreezeFrameState>()(
	persist(
		(set, get) => ({
			isOpen: false,
			lastDurationSeconds: DEFAULT_FREEZE_DURATION_SECONDS,
			resolver: null,

			requestDuration: () =>
				new Promise<number | null>((resolve) => {
					// A new invocation supersedes any pending one: resolve it as
					// cancelled so its handler aborts instead of hanging.
					get().resolver?.({ seconds: null });
					set({
						isOpen: true,
						resolver: ({ seconds }) => resolve(seconds),
					});
				}),
			confirmDuration: ({ seconds }) => {
				if (!isValidFreezeDuration({ seconds })) {
					return;
				}
				get().resolver?.({ seconds });
				set({
					isOpen: false,
					resolver: null,
					lastDurationSeconds: seconds,
				});
			},
			cancelDuration: () => {
				get().resolver?.({ seconds: null });
				set({ isOpen: false, resolver: null });
			},
		}),
		{
			name: "freeze-frame-store",
			partialize: (state) => ({
				lastDurationSeconds: state.lastDurationSeconds,
			}),
		},
	),
);
