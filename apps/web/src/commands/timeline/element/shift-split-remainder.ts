import { Command, type CommandResult } from "@/commands/base-command";
import type { MediaTime } from "@/wasm";
import { MoveElementCommand } from "./move-elements";
import type { SplitElementsCommand } from "./split-elements";

/**
 * Moves the right-side elements produced by a SplitElementsCommand to a new
 * start time. Must run after the split inside a BatchCommand: the right-side
 * element ids only exist once the split has executed. Rebuilds the inner move
 * on every execute() so redo picks up the fresh ids a re-executed split
 * generates.
 */
export class ShiftSplitRemainderCommand extends Command {
	private inner: MoveElementCommand | null = null;

	constructor({
		split,
		newStartTime,
	}: {
		split: SplitElementsCommand;
		newStartTime: MediaTime;
	}) {
		super();
		this.split = split;
		this.newStartTime = newStartTime;
	}

	private readonly split: SplitElementsCommand;
	private readonly newStartTime: MediaTime;

	execute(): CommandResult | undefined {
		const rightSideElements = this.split.getRightSideElements();
		if (rightSideElements.length === 0) {
			return undefined;
		}
		this.inner = new MoveElementCommand({
			moves: rightSideElements.map((element) => ({
				sourceTrackId: element.trackId,
				targetTrackId: element.trackId,
				elementId: element.elementId,
				newStartTime: this.newStartTime,
			})),
		});
		return this.inner.execute();
	}

	undo(): void {
		this.inner?.undo();
	}
}
