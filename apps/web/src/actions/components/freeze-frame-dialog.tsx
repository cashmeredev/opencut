"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogBody,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	isValidFreezeDuration,
	MAX_FREEZE_DURATION_SECONDS,
	useFreezeFrameStore,
} from "@/actions/freeze-frame-store";

export function FreezeFrameDialog() {
	const isOpen = useFreezeFrameStore((state) => state.isOpen);
	const cancelDuration = useFreezeFrameStore((state) => state.cancelDuration);

	return (
		<Dialog
			open={isOpen}
			onOpenChange={(open) => {
				if (!open) {
					cancelDuration();
				}
			}}
		>
			<DialogContent className="sm:max-w-md">
				<DialogHeader>
					<DialogTitle>Freeze frame</DialogTitle>
					<DialogDescription>
						How long should the frozen frame stay on screen?
					</DialogDescription>
				</DialogHeader>
				{isOpen && <FreezeFrameForm />}
			</DialogContent>
		</Dialog>
	);
}

function FreezeFrameForm() {
	const lastDurationSeconds = useFreezeFrameStore(
		(state) => state.lastDurationSeconds,
	);
	const confirmDuration = useFreezeFrameStore((state) => state.confirmDuration);
	const cancelDuration = useFreezeFrameStore((state) => state.cancelDuration);
	const [value, setValue] = useState(String(lastDurationSeconds));

	const seconds = Number(value);
	const showError = value.trim() !== "" && !isValidFreezeDuration({ seconds });
	const canConfirm = isValidFreezeDuration({ seconds });

	const handleConfirm = () => {
		if (canConfirm) {
			confirmDuration({ seconds });
		}
	};

	return (
		<>
			<DialogBody className="gap-3">
				<Label htmlFor="freeze-frame-duration">Duration (seconds)</Label>
				<Input
					id="freeze-frame-duration"
					value={value}
					inputMode="decimal"
					aria-invalid={showError}
					onChange={(e) => setValue(e.target.value)}
					onKeyDown={(e) => {
						if (e.key === "Enter") {
							e.preventDefault();
							handleConfirm();
						}
					}}
					placeholder="e.g. 2.5"
				/>
				{showError && (
					<p className="text-destructive text-xs">
						Enter a duration greater than 0 and at most{" "}
						{MAX_FREEZE_DURATION_SECONDS} seconds.
					</p>
				)}
			</DialogBody>

			<DialogFooter>
				<Button
					variant="outline"
					onClick={(e) => {
						e.preventDefault();
						e.stopPropagation();
						cancelDuration();
					}}
				>
					Cancel
				</Button>
				<Button onClick={handleConfirm} disabled={!canConfirm}>
					Freeze
				</Button>
			</DialogFooter>
		</>
	);
}
