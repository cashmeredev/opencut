import { beforeEach, describe, expect, test } from "bun:test";
import {
	DEFAULT_FREEZE_DURATION_SECONDS,
	isValidFreezeDuration,
	MAX_FREEZE_DURATION_SECONDS,
	useFreezeFrameStore,
} from "../freeze-frame-store";

function resetStore() {
	useFreezeFrameStore.setState({
		isOpen: false,
		lastDurationSeconds: DEFAULT_FREEZE_DURATION_SECONDS,
		resolver: null,
	});
}

describe("freeze-frame store", () => {
	beforeEach(() => {
		resetStore();
	});

	describe("isValidFreezeDuration", () => {
		test("accepts positive durations up to the max, including decimals", () => {
			expect(isValidFreezeDuration({ seconds: 3 })).toBe(true);
			expect(isValidFreezeDuration({ seconds: 2.5 })).toBe(true);
			expect(isValidFreezeDuration({ seconds: 0.1 })).toBe(true);
			expect(
				isValidFreezeDuration({ seconds: MAX_FREEZE_DURATION_SECONDS }),
			).toBe(true);
		});

		test("rejects non-positive, over-max, and non-finite durations", () => {
			expect(isValidFreezeDuration({ seconds: 0 })).toBe(false);
			expect(isValidFreezeDuration({ seconds: -1 })).toBe(false);
			expect(isValidFreezeDuration({ seconds: 600.1 })).toBe(false);
			expect(isValidFreezeDuration({ seconds: Number.NaN })).toBe(false);
			expect(isValidFreezeDuration({ seconds: Number.POSITIVE_INFINITY })).toBe(
				false,
			);
		});
	});

	describe("requestDuration", () => {
		test("opens the dialog and resolves with the confirmed seconds", async () => {
			const promise = useFreezeFrameStore.getState().requestDuration();
			expect(useFreezeFrameStore.getState().isOpen).toBe(true);

			useFreezeFrameStore.getState().confirmDuration({ seconds: 5 });

			await expect(promise).resolves.toBe(5);
			expect(useFreezeFrameStore.getState().isOpen).toBe(false);
		});

		test("remembers the confirmed duration as the last-used value", async () => {
			const promise = useFreezeFrameStore.getState().requestDuration();
			useFreezeFrameStore.getState().confirmDuration({ seconds: 5 });
			await promise;

			expect(useFreezeFrameStore.getState().lastDurationSeconds).toBe(5);
		});

		test("resolves null on cancel and keeps the last-used duration", async () => {
			useFreezeFrameStore.setState({ lastDurationSeconds: 7 });

			const promise = useFreezeFrameStore.getState().requestDuration();
			expect(useFreezeFrameStore.getState().isOpen).toBe(true);

			useFreezeFrameStore.getState().cancelDuration();

			await expect(promise).resolves.toBeNull();
			expect(useFreezeFrameStore.getState().isOpen).toBe(false);
			expect(useFreezeFrameStore.getState().lastDurationSeconds).toBe(7);
		});

		test("ignores an invalid confirm and leaves the dialog open", () => {
			useFreezeFrameStore.getState().requestDuration();

			useFreezeFrameStore.getState().confirmDuration({ seconds: 0 });
			useFreezeFrameStore.getState().confirmDuration({ seconds: 601 });

			expect(useFreezeFrameStore.getState().isOpen).toBe(true);
			expect(useFreezeFrameStore.getState().lastDurationSeconds).toBe(
				DEFAULT_FREEZE_DURATION_SECONDS,
			);
		});

		test("a new request supersedes a pending one by cancelling it", async () => {
			const first = useFreezeFrameStore.getState().requestDuration();
			const second = useFreezeFrameStore.getState().requestDuration();

			await expect(first).resolves.toBeNull();
			expect(useFreezeFrameStore.getState().isOpen).toBe(true);

			useFreezeFrameStore.getState().confirmDuration({ seconds: 2 });
			await expect(second).resolves.toBe(2);
		});
	});
});
