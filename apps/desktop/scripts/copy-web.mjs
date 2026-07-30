import { cpSync, existsSync, rmSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const desktopDir = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const source = path.join(desktopDir, "..", "web", "out");
const target = path.join(desktopDir, "web");

if (!existsSync(source)) {
	process.stderr.write(
		`Web bundle not found at ${source}. Run \`bun build:web\` first.\n`,
	);
	process.exit(1);
}

rmSync(target, { recursive: true, force: true });
cpSync(source, target, { recursive: true });
process.stdout.write(`Copied ${source} -> ${target}\n`);
