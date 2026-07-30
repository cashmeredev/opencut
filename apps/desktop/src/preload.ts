import { contextBridge, ipcRenderer } from "electron";
import type { DesktopExportBegin } from "./export.js";

const api = {
	isDesktop: true as const,
	exportBegin: (
		opts: DesktopExportBegin,
	): Promise<{ id: string; filePath: string } | null> =>
		ipcRenderer.invoke("export:begin", opts),
	exportWriteAudio: (id: string, pcm: ArrayBuffer): Promise<void> =>
		ipcRenderer.invoke("export:writeAudio", id, pcm),
	exportWriteFrame: (id: string, rgba: ArrayBuffer): Promise<void> =>
		ipcRenderer.invoke("export:writeFrame", id, rgba),
	exportFinish: (id: string): Promise<void> =>
		ipcRenderer.invoke("export:finish", id),
	exportCancel: (id: string): Promise<void> =>
		ipcRenderer.invoke("export:cancel", id),
	revealFile: (path: string): Promise<void> =>
		ipcRenderer.invoke("reveal-file", path),
};

contextBridge.exposeInMainWorld("opencutDesktop", api);
