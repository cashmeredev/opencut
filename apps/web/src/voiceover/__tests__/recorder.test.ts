import { describe, expect, test } from "bun:test";
import {
	formatElapsedSeconds,
	getFileExtensionForMimeType,
	getVoiceOverErrorMessage,
	pickSupportedMimeType,
	PREFERRED_MIME_TYPES,
} from "@/voiceover/recorder";

describe("pickSupportedMimeType", () => {
	test("prefers webm/opus when everything is supported", () => {
		const mimeType = pickSupportedMimeType({
			isTypeSupported: () => true,
		});
		expect(mimeType).toBe("audio/webm;codecs=opus");
	});

	test("falls back to the first supported candidate", () => {
		const mimeType = pickSupportedMimeType({
			isTypeSupported: (candidate) => candidate === "audio/mp4",
		});
		expect(mimeType).toBe("audio/mp4");
	});

	test("returns null when nothing is supported", () => {
		const mimeType = pickSupportedMimeType({
			isTypeSupported: () => false,
		});
		expect(mimeType).toBeNull();
	});

	test("respects custom candidate order", () => {
		const mimeType = pickSupportedMimeType({
			isTypeSupported: () => true,
			candidates: ["audio/ogg", "audio/webm"],
		});
		expect(mimeType).toBe("audio/ogg");
	});

	test("default candidates put webm before mp4", () => {
		expect(PREFERRED_MIME_TYPES.indexOf("audio/webm;codecs=opus")).toBeLessThan(
			PREFERRED_MIME_TYPES.indexOf("audio/mp4"),
		);
	});
});

describe("getFileExtensionForMimeType", () => {
	test("maps known audio mime types", () => {
		expect(getFileExtensionForMimeType({ mimeType: "audio/webm" })).toBe(
			"webm",
		);
		expect(
			getFileExtensionForMimeType({ mimeType: "audio/webm;codecs=opus" }),
		).toBe("webm");
		expect(getFileExtensionForMimeType({ mimeType: "audio/ogg" })).toBe("ogg");
		expect(getFileExtensionForMimeType({ mimeType: "audio/mp4" })).toBe("m4a");
		expect(getFileExtensionForMimeType({ mimeType: "audio/mpeg" })).toBe(
			"mp3",
		);
	});

	test("falls back to a generic extension for unknown types", () => {
		expect(getFileExtensionForMimeType({ mimeType: "audio/flac" })).toBe(
			"audio",
		);
		expect(getFileExtensionForMimeType({ mimeType: "" })).toBe("audio");
	});
});

describe("formatElapsedSeconds", () => {
	test("formats zero and sub-second values", () => {
		expect(formatElapsedSeconds({ seconds: 0 })).toBe("0:00");
		expect(formatElapsedSeconds({ seconds: 0.9 })).toBe("0:00");
	});

	test("pads seconds", () => {
		expect(formatElapsedSeconds({ seconds: 9.6 })).toBe("0:09");
	});

	test("formats minutes", () => {
		expect(formatElapsedSeconds({ seconds: 65 })).toBe("1:05");
		expect(formatElapsedSeconds({ seconds: 600 })).toBe("10:00");
	});

	test("clamps negative values", () => {
		expect(formatElapsedSeconds({ seconds: -3 })).toBe("0:00");
	});
});

describe("getVoiceOverErrorMessage", () => {
	test("explains denied permission", () => {
		const message = getVoiceOverErrorMessage({
			error: new DOMException("denied", "NotAllowedError"),
		});
		expect(message).toContain("Microphone access denied");
	});

	test("explains a missing microphone", () => {
		const message = getVoiceOverErrorMessage({
			error: new DOMException("missing", "NotFoundError"),
		});
		expect(message).toContain("No microphone found");
	});

	test("explains a busy microphone", () => {
		const message = getVoiceOverErrorMessage({
			error: new DOMException("busy", "NotReadableError"),
		});
		expect(message).toContain("already in use");
	});

	test("surfaces generic error messages", () => {
		const message = getVoiceOverErrorMessage({
			error: new Error("Voice-over recording is not supported in this browser"),
		});
		expect(message).toBe(
			"Voice-over recording is not supported in this browser",
		);
	});

	test("handles non-error values", () => {
		expect(getVoiceOverErrorMessage({ error: null })).toBe(
			"Could not start voice-over recording.",
		);
	});
});
