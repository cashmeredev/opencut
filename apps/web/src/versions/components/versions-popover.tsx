"use client";

import { useState } from "react";
import Image from "next/image";
import {
	Clock01Icon,
	Delete02Icon,
	Edit02Icon,
	PlusSignIcon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from "@/components/ui/popover";
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useEditor } from "@/editor/use-editor";
import { useVersionsStore } from "@/versions/versions-store";
import { VersionNameDialog } from "@/versions/components/version-name-dialog";
import type { ProjectVersion } from "@/versions/types";

function formatVersionDate({ date }: { date: Date }): string {
	return new Date(date).toLocaleString(undefined, {
		dateStyle: "medium",
		timeStyle: "short",
	});
}

function getVersionTitle({ version }: { version: ProjectVersion }): string {
	return version.kind === "named" && version.name
		? version.name
		: "Auto-checkpoint";
}

export function VersionsButton() {
	const editor = useEditor();
	const isPanelOpen = useVersionsStore((s) => s.isPanelOpen);
	const setPanelOpen = useVersionsStore((s) => s.setPanelOpen);
	const isCheckpointDialogOpen = useVersionsStore(
		(s) => s.isCheckpointDialogOpen,
	);
	const closeCheckpointDialog = useVersionsStore(
		(s) => s.closeCheckpointDialog,
	);
	const openCheckpointDialog = useVersionsStore((s) => s.openCheckpointDialog);
	const projectId = useEditor((e) => e.project.getActiveOrNull()?.metadata.id);
	const versions = useEditor((e) =>
		projectId ? e.versions.getVersions({ projectId }) : [],
	);

	const [restoreTarget, setRestoreTarget] = useState<ProjectVersion | null>(
		null,
	);
	const [renameTarget, setRenameTarget] = useState<ProjectVersion | null>(null);
	const [deleteTarget, setDeleteTarget] = useState<ProjectVersion | null>(null);

	const handlePanelOpenChange = (open: boolean) => {
		setPanelOpen({ open });
		if (open && projectId) {
			editor.versions.listVersions({ projectId }).catch((error) => {
				console.error("Failed to refresh versions:", error);
			});
		}
	};

	const handleCreateCheckpoint = (name: string) => {
		closeCheckpointDialog();
		editor.versions.createNamedCheckpoint({ name }).catch((error) => {
			console.error("Failed to create checkpoint:", error);
		});
	};

	return (
		<>
			<Popover open={isPanelOpen} onOpenChange={handlePanelOpenChange}>
				<PopoverTrigger asChild>
					<Button
						variant="ghost"
						size="icon"
						className="size-8"
						title="Versions"
					>
						<HugeiconsIcon icon={Clock01Icon} className="size-4" />
					</Button>
				</PopoverTrigger>
				<PopoverContent align="end" className="w-80 p-2">
					<div className="flex items-center justify-between px-2 pb-2">
						<span className="text-sm font-medium">Versions</span>
						<Button
							variant="outline"
							size="sm"
							onClick={openCheckpointDialog}
						>
							<HugeiconsIcon icon={PlusSignIcon} className="size-3.5" />
							Checkpoint
						</Button>
					</div>
					{versions.length === 0 ? (
						<p className="text-muted-foreground px-2 py-6 text-center text-sm">
							No versions yet. Create a checkpoint to save a known-good state.
						</p>
					) : (
						<ScrollArea className="max-h-80">
							<div className="flex flex-col gap-1">
								{versions.map((version) => (
									<VersionRow
										key={version.id}
										version={version}
										onRestore={() => setRestoreTarget(version)}
										onRename={
											version.kind === "named"
												? () => setRenameTarget(version)
												: undefined
										}
										onDelete={
											version.kind === "named"
												? () => setDeleteTarget(version)
												: undefined
										}
									/>
								))}
							</div>
						</ScrollArea>
					)}
				</PopoverContent>
			</Popover>

			<VersionNameDialog
				isOpen={isCheckpointDialogOpen}
				onOpenChange={(open) => {
					if (!open) closeCheckpointDialog();
				}}
				onConfirm={handleCreateCheckpoint}
				mode="create"
			/>

			<VersionNameDialog
				isOpen={renameTarget !== null}
				onOpenChange={(open) => {
					if (!open) setRenameTarget(null);
				}}
				onConfirm={(name) => {
					const target = renameTarget;
					setRenameTarget(null);
					if (target) {
						editor.versions
							.renameVersion({ versionId: target.id, name })
							.catch((error) => {
								console.error("Failed to rename checkpoint:", error);
							});
					}
				}}
				mode="rename"
				initialName={renameTarget?.name ?? ""}
			/>

			<AlertDialog
				open={restoreTarget !== null}
				onOpenChange={(open) => {
					if (!open) setRestoreTarget(null);
				}}
			>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>
							Restore {restoreTarget ? getVersionTitle({ version: restoreTarget }) : ""}?
						</AlertDialogTitle>
						<AlertDialogDescription>
							The editor will jump to this version. Your current state is saved
							as an auto-checkpoint first, so nothing is lost.
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel>Cancel</AlertDialogCancel>
						<AlertDialogAction
							onClick={() => {
								const target = restoreTarget;
								setRestoreTarget(null);
								if (target) {
									editor.versions
										.restoreVersion({ versionId: target.id })
										.catch((error) => {
											console.error("Failed to restore version:", error);
										});
								}
							}}
						>
							Restore
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>

			<AlertDialog
				open={deleteTarget !== null}
				onOpenChange={(open) => {
					if (!open) setDeleteTarget(null);
				}}
			>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>
							Delete {deleteTarget ? getVersionTitle({ version: deleteTarget }) : ""}?
						</AlertDialogTitle>
						<AlertDialogDescription>
							This checkpoint is removed permanently. The current state of the
							project is not affected.
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel>Cancel</AlertDialogCancel>
						<AlertDialogAction
							onClick={() => {
								const target = deleteTarget;
								setDeleteTarget(null);
								if (target) {
									editor.versions
										.deleteVersion({ versionId: target.id })
										.catch((error) => {
											console.error("Failed to delete checkpoint:", error);
										});
								}
							}}
						>
							Delete
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>
		</>
	);
}

function VersionRow({
	version,
	onRestore,
	onRename,
	onDelete,
}: {
	version: ProjectVersion;
	onRestore: () => void;
	onRename?: () => void;
	onDelete?: () => void;
}) {
	return (
		<div className="hover:bg-accent flex items-center gap-1 rounded-sm p-1">
			<Button
				variant="ghost"
				className="h-auto flex-1 justify-start gap-2 px-1 py-1 text-left font-normal"
				onClick={onRestore}
			>
				<div className="bg-muted relative flex size-12 shrink-0 items-center justify-center overflow-hidden rounded-sm">
					{version.thumbnail ? (
						<Image
							src={version.thumbnail}
							alt=""
							fill
							sizes="48px"
							className="object-cover"
						/>
					) : (
						<HugeiconsIcon
							icon={Clock01Icon}
							className="text-muted-foreground size-4"
						/>
					)}
				</div>
				<div className="flex min-w-0 flex-col gap-0.5">
					<span className="truncate text-sm">
						{getVersionTitle({ version })}
					</span>
					<span className="text-muted-foreground text-xs">
						{formatVersionDate({ date: version.createdAt })}
					</span>
				</div>
				<Badge
					variant={version.kind === "named" ? "default" : "secondary"}
					className="ml-auto shrink-0"
				>
					{version.kind === "named" ? "Named" : "Auto"}
				</Badge>
			</Button>
			{onRename && (
				<Button
					variant="ghost"
					size="icon"
					className="size-7 shrink-0"
					title="Rename checkpoint"
					onClick={onRename}
				>
					<HugeiconsIcon icon={Edit02Icon} className="size-3.5" />
				</Button>
			)}
			{onDelete && (
				<Button
					variant="ghost"
					size="icon"
					className="size-7 shrink-0"
					title="Delete checkpoint"
					onClick={onDelete}
				>
					<HugeiconsIcon icon={Delete02Icon} className="size-3.5" />
				</Button>
			)}
		</div>
	);
}
