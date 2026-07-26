import { ACTIONS } from "./definitions";
import type { TActionArgsMap, TActionWithOptionalArgs } from "./types";

type TActionWithRequiredArgs = {
	[K in keyof TActionArgsMap]: undefined extends TActionArgsMap[K]
		? never
		: K;
}[keyof TActionArgsMap];

/**
 * Actions that require arguments cannot be bound to a shortcut key — a key
 * press has no way to supply them. Keep in sync with the required (non-
 * optional) members of `TActionArgsMap`; the type annotation rejects keys
 * that are not actually required-args actions.
 */
const ACTIONS_WITH_REQUIRED_ARGS: readonly TActionWithRequiredArgs[] = [
	"remove-media-asset",
	"remove-media-assets",
];

export function isActionWithOptionalArgs(
	action: unknown,
): action is TActionWithOptionalArgs {
	return (
		typeof action === "string" &&
		action in ACTIONS &&
		!(ACTIONS_WITH_REQUIRED_ARGS as readonly string[]).includes(action)
	);
}
