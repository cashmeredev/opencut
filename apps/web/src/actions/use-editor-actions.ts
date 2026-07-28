"use client";

import { useEffect, useState } from "react";
import { useTimelineStore } from "@/timeline/timeline-store";
import { useActionHandler } from "@/actions/use-action-handler";
import { useEditor } from "@/editor/use-editor";
import { useElementSelection } from "@/timeline/hooks/element/use-element-selection";
import {
	addMediaTime,
	maxMediaTime,
	mediaTime,
	mediaTimeFromSeconds,
	minMediaTime,
	subMediaTime,
	TICKS_PER_SECOND,
	ZERO_MEDIA_TIME,
} from "@/wasm";
import { useKeyframeSelection } from "@/timeline/hooks/element/use-keyframe-selection";
import { getElementsAtTime, hasMediaId, buildElementFromMedia } from "@/timeline";
import { cancelInteraction } from "@/editor/cancel-interaction";
import { invokeAction } from "@/actions";
import { canToggleSourceAudio } from "@/timeline/audio-separation";
import {
	activateScope,
	clearActiveScope,
	type ScopeEntry,
} from "@/selection/scope";
import { useCommittedRef } from "@/hooks/use-committed-ref";
import { toast } from "sonner";
import { generateImageThumbnail } from "@/media/processing";
import {
	AddMediaAssetCommand,
	BatchCommand,
	InsertElementCommand,
	ShiftSplitRemainderCommand,
	SplitElementsCommand,
} from "@/commands";
import { useVersionsStore } from "@/versions/versions-store";
import {
	cancelVoiceOverRecording,
	getFileExtensionForMimeType,
	getVoiceOverErrorMessage,
	startVoiceOverRecording,
	stopVoiceOverRecording,
} from "@/voiceover/recorder";
import type { VoiceOverRecordingResult } from "@/voiceover/recorder";
import { useVoiceOverStore } from "@/voiceover/voiceover-store";

const FREEZE_DURATION_SECONDS = 3;

export function useEditorActions() {
	const editor = useEditor();
	const { selectedElements, setElementSelection } = useElementSelection();
	const { selectedKeyframes, clearKeyframeSelection } = useKeyframeSelection();
	const selectedMaskPointSelection = useEditor((e) =>
		e.selection.getSelectedMaskPointSelection(),
	);
	const toggleSnapping = useTimelineStore((s) => s.toggleSnapping);
	const rippleEditingEnabled = useTimelineStore((s) => s.rippleEditingEnabled);
	const toggleRippleEditing = useTimelineStore((s) => s.toggleRippleEditing);
	const openCheckpointDialog = useVersionsStore((s) => s.openCheckpointDialog);
	const toggleVersionsPanel = useVersionsStore((s) => s.togglePanel);
	const hasTimelineSelection =
		selectedElements.length > 0 ||
		selectedKeyframes.length > 0 ||
		selectedMaskPointSelection !== null;
	const hasTimelineSelectionRef = useCommittedRef(hasTimelineSelection);
	const clearTimelineSelectionRef = useCommittedRef(() => {
		editor.selection.clearSelection();
	});
	const clearTimelineActiveSelectionRef = useCommittedRef(() => {
		editor.selection.clearMostSpecificSelection();
	});
	const [timelineScope] = useState<ScopeEntry>(() => ({
		hasSelection: () => hasTimelineSelectionRef.current,
		clear: () => {
			clearTimelineSelectionRef.current();
		},
		clearActive: () => {
			clearTimelineActiveSelectionRef.current();
		},
	}));

	useEffect(() => {
		if (!hasTimelineSelection) {
			return;
		}

		return activateScope({ entry: timelineScope });
	}, [hasTimelineSelection, timelineScope]);

	useActionHandler(
		"toggle-play",
		() => {
			editor.playback.toggle();
		},
		undefined,
	);

	useActionHandler(
		"stop-playback",
		() => {
			if (editor.playback.getIsPlaying()) {
				editor.playback.toggle();
			}
			editor.playback.seek({ time: ZERO_MEDIA_TIME });
		},
		undefined,
	);

	useActionHandler(
		"seek-forward",
		(args) => {
			const seconds = args?.seconds ?? 1;
			const delta = mediaTimeFromSeconds({ seconds });
			editor.playback.seek({
				time: minMediaTime({
					a: editor.timeline.getTotalDuration(),
					b: addMediaTime({
						a: editor.playback.getCurrentTime(),
						b: delta,
					}),
				}),
			});
		},
		undefined,
	);

	useActionHandler(
		"seek-backward",
		(args) => {
			const seconds = args?.seconds ?? 1;
			const delta = mediaTimeFromSeconds({ seconds });
			editor.playback.seek({
				time: maxMediaTime({
					a: ZERO_MEDIA_TIME,
					b: subMediaTime({
						a: editor.playback.getCurrentTime(),
						b: delta,
					}),
				}),
			});
		},
		undefined,
	);

	useActionHandler(
		"frame-step-forward",
		() => {
			const fps = editor.project.getActive().settings.fps;
			const ticksPerFrame = mediaTime({
				ticks: Math.round(
					(TICKS_PER_SECOND * fps.denominator) / fps.numerator,
				),
			});
			editor.playback.seek({
				time: minMediaTime({
					a: editor.timeline.getTotalDuration(),
					b: addMediaTime({
						a: editor.playback.getCurrentTime(),
						b: ticksPerFrame,
					}),
				}),
			});
		},
		undefined,
	);

	useActionHandler(
		"frame-step-backward",
		() => {
			const fps = editor.project.getActive().settings.fps;
			const ticksPerFrame = mediaTime({
				ticks: Math.round(
					(TICKS_PER_SECOND * fps.denominator) / fps.numerator,
				),
			});
			editor.playback.seek({
				time: maxMediaTime({
					a: ZERO_MEDIA_TIME,
					b: subMediaTime({
						a: editor.playback.getCurrentTime(),
						b: ticksPerFrame,
					}),
				}),
			});
		},
		undefined,
	);

	useActionHandler(
		"jump-forward",
		(args) => {
			const seconds = args?.seconds ?? 5;
			const delta = mediaTimeFromSeconds({ seconds });
			editor.playback.seek({
				time: minMediaTime({
					a: editor.timeline.getTotalDuration(),
					b: addMediaTime({
						a: editor.playback.getCurrentTime(),
						b: delta,
					}),
				}),
			});
		},
		undefined,
	);

	useActionHandler(
		"jump-backward",
		(args) => {
			const seconds = args?.seconds ?? 5;
			const delta = mediaTimeFromSeconds({ seconds });
			editor.playback.seek({
				time: maxMediaTime({
					a: ZERO_MEDIA_TIME,
					b: subMediaTime({
						a: editor.playback.getCurrentTime(),
						b: delta,
					}),
				}),
			});
		},
		undefined,
	);

	useActionHandler(
		"goto-start",
		() => {
			editor.playback.seek({ time: ZERO_MEDIA_TIME });
		},
		undefined,
	);

	useActionHandler(
		"goto-end",
		() => {
			editor.playback.seek({ time: editor.timeline.getTotalDuration() });
		},
		undefined,
	);

	useActionHandler(
		"split",
		() => {
			const currentTime = editor.playback.getCurrentTime();
			const tracks = editor.scenes.getActiveScene().tracks;
			const elementsToSplit =
				selectedElements.length > 0
					? selectedElements
					: getElementsAtTime({
							tracks,
							time: currentTime,
						});

			if (elementsToSplit.length === 0) return;

			editor.timeline.splitElements({
				elements: elementsToSplit,
				splitTime: currentTime,
			});
		},
		undefined,
	);

	useActionHandler(
		"split-left",
		() => {
			const currentTime = editor.playback.getCurrentTime();
			const tracks = editor.scenes.getActiveScene().tracks;
			const elementsToSplit =
				selectedElements.length > 0
					? selectedElements
					: getElementsAtTime({
							tracks,
							time: currentTime,
						});

			if (elementsToSplit.length === 0) return;

			const rightSideElements = editor.timeline.splitElements({
				elements: elementsToSplit,
				splitTime: currentTime,
				retainSide: "right",
			});

			if (rippleEditingEnabled && rightSideElements.length > 0) {
				const firstRightElement = editor.timeline.getElementsWithTracks({
					elements: [rightSideElements[0]],
				})[0];
				if (firstRightElement) {
					editor.playback.seek({ time: firstRightElement.element.startTime });
				}
			}
		},
		undefined,
	);

	useActionHandler(
		"split-right",
		() => {
			const currentTime = editor.playback.getCurrentTime();
			const tracks = editor.scenes.getActiveScene().tracks;
			const elementsToSplit =
				selectedElements.length > 0
					? selectedElements
					: getElementsAtTime({
							tracks,
							time: currentTime,
						});

			if (elementsToSplit.length === 0) return;

			editor.timeline.splitElements({
				elements: elementsToSplit,
				splitTime: currentTime,
				retainSide: "left",
			});
		},
		undefined,
	);

	useActionHandler(
		"delete-selected",
		() => {
			switch (editor.selection.getActiveSelectionKind()) {
				case "mask-points":
					if (!selectedMaskPointSelection) {
						return;
					}
					editor.timeline.deleteFreeformPathMaskPoints({
						trackId: selectedMaskPointSelection.trackId,
						elementId: selectedMaskPointSelection.elementId,
						maskId: selectedMaskPointSelection.maskId,
						pointIds: selectedMaskPointSelection.pointIds,
					});
					return;
				case "keyframes":
					if (selectedKeyframes.length === 0) {
						return;
					}
					editor.timeline.removeKeyframes({ keyframes: selectedKeyframes });
					clearKeyframeSelection();
					return;
				case "elements":
					if (selectedElements.length === 0) {
						return;
					}
					editor.timeline.deleteElements({
						elements: selectedElements,
					});
					return;
				default:
					return;
			}
		},
		undefined,
	);

	useActionHandler(
		"toggle-source-audio",
		() => {
			if (selectedElements.length !== 1) {
				return;
			}

			const selectedElement = editor.timeline.getElementsWithTracks({
				elements: selectedElements,
			})[0];
			if (!selectedElement) {
				return;
			}

			const mediaAsset = (() => {
				const { element } = selectedElement;
				if (!hasMediaId(element)) {
					return null;
				}

				return (
					editor.media
						.getAssets()
						.find((asset) => asset.id === element.mediaId) ?? null
				);
			})();
			if (!canToggleSourceAudio(selectedElement.element, mediaAsset)) {
				return;
			}

			editor.timeline.toggleSourceAudioSeparation({
				trackId: selectedElement.track.id,
				elementId: selectedElement.element.id,
			});
		},
		undefined,
	);

	useActionHandler(
		"freeze-frame",
		async () => {
			const currentTime = editor.playback.getCurrentTime();
			const tracks = editor.scenes.getActiveScene().tracks;
			const candidates =
				selectedElements.length > 0
					? selectedElements
					: getElementsAtTime({ tracks, time: currentTime });

			const target = editor.timeline
				.getElementsWithTracks({ elements: candidates })
				.find(
					({ element }) =>
						element.type === "video" &&
						currentTime > element.startTime &&
						currentTime < element.startTime + element.duration,
				);
			if (!target) {
				return;
			}

			const activeProject = editor.project.getActive();
			if (!activeProject) {
				return;
			}

			const blob = await editor.renderer.captureFrame();
			if (!blob) {
				toast.error("Failed to capture frame");
				return;
			}

			const file = new File([blob], "freeze-frame.png", {
				type: "image/png",
			});
			const { thumbnailUrl } = await generateImageThumbnail({
				imageFile: file,
			});

			const freezeDuration = mediaTimeFromSeconds({
				seconds: FREEZE_DURATION_SECONDS,
			});
			const addMediaCommand = new AddMediaAssetCommand({
				projectId: activeProject.metadata.id,
				asset: {
					file,
					name: "Freeze frame",
					type: "image",
					url: URL.createObjectURL(file),
					thumbnailUrl,
					width: activeProject.settings.canvasSize.width,
					height: activeProject.settings.canvasSize.height,
				},
			});
			const splitCommand = new SplitElementsCommand({
				elements: [
					{ trackId: target.track.id, elementId: target.element.id },
				],
				splitTime: currentTime,
			});
			const shiftCommand = new ShiftSplitRemainderCommand({
				split: splitCommand,
				newStartTime: addMediaTime({ a: currentTime, b: freezeDuration }),
			});
			const insertCommand = new InsertElementCommand({
				element: buildElementFromMedia({
					mediaId: addMediaCommand.getAssetId(),
					mediaType: "image",
					name: "Freeze frame",
					duration: freezeDuration,
					startTime: currentTime,
				}),
				placement: { mode: "explicit", trackId: target.track.id },
			});
			editor.command.execute({
				command: new BatchCommand([
					addMediaCommand,
					splitCommand,
					shiftCommand,
					insertCommand,
				]),
			});
		},
		undefined,
	);

	useActionHandler(
		"select-all",
		() => {
			const scene = editor.scenes.getActiveScene();
			const allElements = [
				...scene.tracks.overlay,
				scene.tracks.main,
				...scene.tracks.audio,
			].flatMap((track) =>
				track.elements.map((element) => ({
					trackId: track.id,
					elementId: element.id,
				})),
			);
			setElementSelection({ elements: allElements });
		},
		undefined,
	);

	useActionHandler(
		"cancel-interaction",
		() => {
			if (!cancelInteraction()) {
				invokeAction("deselect-all");
			}
		},
		undefined,
	);

	useActionHandler(
		"deselect-all",
		() => {
			if (!clearActiveScope()) {
				editor.selection.clearMostSpecificSelection();
			}
		},
		undefined,
	);

	useActionHandler(
		"duplicate-selected",
		() => {
			editor.timeline.duplicateElements({
				elements: selectedElements,
			});
		},
		undefined,
	);

	useActionHandler(
		"toggle-elements-muted-selected",
		() => {
			editor.timeline.toggleElementsMuted({ elements: selectedElements });
		},
		undefined,
	);

	useActionHandler(
		"toggle-elements-visibility-selected",
		() => {
			editor.timeline.toggleElementsVisibility({ elements: selectedElements });
		},
		undefined,
	);

	useActionHandler(
		"toggle-bookmark",
		() => {
			editor.scenes.toggleBookmark({ time: editor.playback.getCurrentTime() });
		},
		undefined,
	);

	useActionHandler(
		"copy-selected",
		() => {
			editor.clipboard.copy();
		},
		undefined,
	);

	useActionHandler(
		"paste-copied",
		() => {
			editor.clipboard.paste();
		},
		undefined,
	);

	useActionHandler(
		"toggle-snapping",
		() => {
			toggleSnapping();
		},
		undefined,
	);

	useActionHandler(
		"toggle-ripple-editing",
		() => {
			toggleRippleEditing();
		},
		undefined,
	);

	useActionHandler(
		"undo",
		() => {
			editor.command.undo();
		},
		undefined,
	);

	useActionHandler(
		"redo",
		() => {
			editor.command.redo();
		},
		undefined,
	);

	// todo: potnetially unify these two actions:
	useActionHandler(
		"remove-media-asset",
		(args) => {
			if (!args) return;
			editor.media.removeMediaAsset({
				projectId: args.projectId,
				id: args.assetId,
			});
		},
		undefined,
	);

	useActionHandler(
		"remove-media-assets",
		(args) => {
			if (!args) return;
			editor.media.removeMediaAssets({
				projectId: args.projectId,
				ids: args.assetIds,
			});
		},
		undefined,
	);

	useActionHandler(
		"create-checkpoint",
		() => {
			openCheckpointDialog();
		},
		undefined,
	);

	useActionHandler(
		"toggle-versions-panel",
		() => {
			toggleVersionsPanel();
		},
		undefined,
	);

	useActionHandler(
		"voice-over",
		async () => {
			const voiceOverState = useVoiceOverStore.getState();

			if (voiceOverState.status === "idle") {
				try {
					await startVoiceOverRecording();
				} catch (error) {
					toast.error(getVoiceOverErrorMessage({ error }));
					return;
				}
				useVoiceOverStore.getState().startRecording({
					playheadTime: editor.playback.getCurrentTime(),
				});
				return;
			}

			useVoiceOverStore.getState().stopRecording();

			const activeProject = editor.project.getActive();
			if (!activeProject) {
				cancelVoiceOverRecording();
				return;
			}

			let recording: VoiceOverRecordingResult;
			try {
				recording = await stopVoiceOverRecording();
			} catch (error) {
				toast.error(getVoiceOverErrorMessage({ error }));
				return;
			}

			if (recording.blob.size === 0) {
				toast.error("Voice-over recording is empty");
				return;
			}

			const file = new File(
				[recording.blob],
				`voice-over.${getFileExtensionForMimeType({ mimeType: recording.mimeType })}`,
				{ type: recording.mimeType },
			);
			const addMediaCommand = new AddMediaAssetCommand({
				projectId: activeProject.metadata.id,
				asset: {
					file,
					name: "Voice over",
					type: "audio",
					url: URL.createObjectURL(file),
					duration: recording.durationSeconds,
				},
			});
			const insertCommand = new InsertElementCommand({
				element: buildElementFromMedia({
					mediaId: addMediaCommand.getAssetId(),
					mediaType: "audio",
					name: "Voice over",
					duration: mediaTimeFromSeconds({
						seconds: recording.durationSeconds,
					}),
					startTime:
						voiceOverState.playheadStartTime ??
						editor.playback.getCurrentTime(),
				}),
				placement: { mode: "auto", trackType: "audio" },
			});
			editor.command.execute({
				command: new BatchCommand([addMediaCommand, insertCommand]),
			});
		},
		undefined,
	);

	useActionHandler(
		"export-project-archive",
		() => {
			const projectId = editor.project.getActiveOrNull()?.metadata.id;
			if (!projectId) return;
			void editor.project.exportArchive({ id: projectId });
		},
		undefined,
	);
}
