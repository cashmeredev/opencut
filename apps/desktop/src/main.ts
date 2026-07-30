import { app, BrowserWindow, protocol } from "electron";
import { stat } from "node:fs/promises";
import { createReadStream } from "node:fs";
import path from "node:path";
import { Readable } from "node:stream";
import { cancelAllExports, registerExportHandlers } from "./export.js";

const contentTypes: Record<string, string> = {
	".html": "text/html; charset=utf-8",
	".js": "text/javascript; charset=utf-8",
	".css": "text/css; charset=utf-8",
	".json": "application/json; charset=utf-8",
	".wasm": "application/wasm",
	".png": "image/png",
	".jpg": "image/jpeg",
	".svg": "image/svg+xml",
	".ico": "image/x-icon",
	".woff2": "font/woff2",
	".mp4": "video/mp4",
	".webm": "video/webm",
	".txt": "text/plain; charset=utf-8",
	".xml": "application/xml; charset=utf-8",
	".map": "application/json; charset=utf-8",
};

protocol.registerSchemesAsPrivileged([
	{
		scheme: "opencut",
		privileges: {
			standard: true,
			secure: true,
			supportFetchAPI: true,
			stream: true,
			bypassCSP: false,
		},
	},
]);

function registerWebProtocol(): void {
	const webDir = app.isPackaged
		? path.join(process.resourcesPath, "web")
		: path.join(__dirname, "..", "web");

	protocol.handle("opencut", async (request) => {
		const url = new URL(request.url);
		const pathname = decodeURIComponent(url.pathname);
		const resolved = path.normalize(path.join(webDir, pathname));

		if (resolved !== webDir && !resolved.startsWith(`${webDir}${path.sep}`)) {
			return new Response("Forbidden", { status: 403 });
		}

		let filePath = resolved;
		let info = await stat(filePath).catch(() => null);

		if (info?.isDirectory() || (!info && path.extname(filePath) === "")) {
			filePath = path.join(resolved, "index.html");
			info = await stat(filePath).catch(() => null);
		}

		if (!info?.isFile()) {
			return new Response("Not found", { status: 404 });
		}

		const contentType = contentTypes[path.extname(filePath).toLowerCase()];
		if (!contentType) {
			return new Response("Not found", { status: 404 });
		}

		const stream = Readable.toWeb(createReadStream(filePath));
		return new Response(stream as ReadableStream, {
			headers: { "Content-Type": contentType },
		});
	});
}

function createWindow(): void {
	const win = new BrowserWindow({
		width: 1440,
		height: 900,
		minWidth: 960,
		minHeight: 600,
		backgroundColor: "#0a0a0a",
		autoHideMenuBar: true,
		webPreferences: {
			preload: path.join(__dirname, "preload.js"),
			contextIsolation: true,
			nodeIntegration: false,
			sandbox: false,
		},
	});

	const devServerUrl = process.env.OPENCUT_DEV_SERVER_URL;
	if (devServerUrl) {
		win.loadURL(devServerUrl);
		return;
	}

	win.loadURL("opencut://app/");
}

const gotSingleInstanceLock = app.requestSingleInstanceLock();

if (!gotSingleInstanceLock) {
	app.quit();
} else {
	app.on("second-instance", () => {
		const [win] = BrowserWindow.getAllWindows();
		if (!win) return;
		if (win.isMinimized()) win.restore();
		win.focus();
	});

	app.whenReady().then(() => {
		registerWebProtocol();
		registerExportHandlers();
		createWindow();

		app.on("activate", () => {
			if (BrowserWindow.getAllWindows().length === 0) createWindow();
		});
	});
}

app.on("window-all-closed", () => {
	app.quit();
});

app.on("will-quit", () => {
	cancelAllExports();
});
