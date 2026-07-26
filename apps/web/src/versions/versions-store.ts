import { create } from "zustand";

/**
 * UI-only state for the versions surfaces (popover + checkpoint dialog).
 * Document state lives in EditorCore's VersionsManager — never here.
 */
interface VersionsState {
	isPanelOpen: boolean;
	isCheckpointDialogOpen: boolean;
	setPanelOpen: (args: { open: boolean }) => void;
	togglePanel: () => void;
	openCheckpointDialog: () => void;
	closeCheckpointDialog: () => void;
}

export const useVersionsStore = create<VersionsState>()((set) => ({
	isPanelOpen: false,
	isCheckpointDialogOpen: false,
	setPanelOpen: ({ open }) => set({ isPanelOpen: open }),
	togglePanel: () => set((state) => ({ isPanelOpen: !state.isPanelOpen })),
	openCheckpointDialog: () => set({ isCheckpointDialogOpen: true }),
	closeCheckpointDialog: () => set({ isCheckpointDialogOpen: false }),
}));
