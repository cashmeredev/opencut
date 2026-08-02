import { effectsRegistry } from "../registry";
import { blurEffectDefinition } from "./blur";
import { motionEffectDefinitions } from "./motion";

const defaultEffects = [blurEffectDefinition, ...motionEffectDefinitions];

export function registerDefaultEffects(): void {
	for (const definition of defaultEffects) {
		if (effectsRegistry.has(definition.type)) {
			continue;
		}
		effectsRegistry.register({
			key: definition.type,
			definition,
		});
	}
}
