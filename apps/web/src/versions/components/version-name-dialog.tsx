import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogBody,
	DialogContent,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { useState } from "react";
import { Label } from "@/components/ui/label";

export function VersionNameDialog({
	isOpen,
	onOpenChange,
	onConfirm,
	mode,
	initialName = "",
}: {
	isOpen: boolean;
	onOpenChange: (open: boolean) => void;
	onConfirm: (name: string) => void;
	mode: "create" | "rename";
	initialName?: string;
}) {
	const [name, setName] = useState(initialName);

	const handleOpenChange = (open: boolean) => {
		if (open) {
			setName(initialName);
		}
		onOpenChange(open);
	};

	const title = mode === "create" ? "Create checkpoint" : "Rename checkpoint";
	const confirmLabel = mode === "create" ? "Create" : "Rename";

	return (
		<Dialog open={isOpen} onOpenChange={handleOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>{title}</DialogTitle>
				</DialogHeader>

				<DialogBody className="gap-3">
					<Label>Name</Label>
					<Input
						value={name}
						onChange={(e) => setName(e.target.value)}
						onKeyDown={(e) => {
							if (e.key === "Enter") {
								e.preventDefault();
								onConfirm(name);
							}
						}}
						placeholder="e.g. Final cut"
					/>
				</DialogBody>

				<DialogFooter>
					<Button
						variant="outline"
						onClick={(e) => {
							e.preventDefault();
							e.stopPropagation();
							onOpenChange(false);
						}}
					>
						Cancel
					</Button>
					<Button onClick={() => onConfirm(name)}>{confirmLabel}</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}
