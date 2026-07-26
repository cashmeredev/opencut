import type { SerializedProject } from "@/services/storage/types";

export type ProjectVersionKind = "named" | "auto";

/**
 * A point-in-time snapshot of a project (see notes/project-versioning.md).
 *
 * Media is referenced by asset id inside `project` (v1 keeps the per-project
 * media store); use `getReferencedMediaIds` to extract the referenced set.
 */
export interface ProjectVersion {
	id: string;
	projectId: string;
	kind: ProjectVersionKind;
	/** Required for kind "named". */
	name?: string;
	createdAt: Date;
	project: SerializedProject;
	/** Data URL captured via renderer.captureFrame(); named versions only. */
	thumbnail?: string;
}
