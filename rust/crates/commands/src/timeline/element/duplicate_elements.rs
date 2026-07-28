use scene::{Element, ElementRef, Project, SceneTracks};
use timeline::TrackType;
use timeline::placement::apply::ApplyPlacementParams;
use timeline::placement::{
    NewTrackPosition, PlacementResult, PlacementStrategy, PlacementSubject,
    ResolveTrackPlacementParams, apply_placement, ordered_tracks, resolve_track_placement,
};
use uuid::Uuid;

use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;

fn build_duplicate_element(element: &Element, id: String) -> Element {
    let mut duplicate = element.clone();
    {
        let base = duplicate.base_mut();
        base.id = id;
        base.name = format!("{} (copy)", base.name);
        base.animations = animation::clone_animations(base.animations.as_ref(), true);
    }
    duplicate
}

pub struct DuplicateElementsCommand {
    elements: Vec<ElementRef>,
    duplicated: Vec<ElementRef>,
    saved_state: Option<SceneTracks>,
}

impl DuplicateElementsCommand {
    pub fn new(elements: Vec<ElementRef>) -> Self {
        Self {
            elements,
            duplicated: Vec::new(),
            saved_state: None,
        }
    }

    pub fn duplicated_elements(&self) -> &[ElementRef] {
        &self.duplicated
    }
}

impl Command for DuplicateElementsCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let Some(scene) = active_scene_mut(project) else {
            return Ok(None);
        };

        self.saved_state = Some(scene.tracks.clone());
        self.duplicated.clear();

        let saved_state = scene.tracks.clone();
        let mut updated_tracks = scene.tracks.clone();

        for track in ordered_tracks(&saved_state) {
            let new_elements: Vec<Element> = track
                .elements()
                .iter()
                .filter(|element| {
                    self.elements.iter().any(|target| {
                        target.track_id == track.id() && target.element_id == element.base().id
                    })
                })
                .map(|element| build_duplicate_element(element, Uuid::new_v4().to_string()))
                .collect();

            if new_elements.is_empty() {
                continue;
            }

            let placement_result = resolve_track_placement(&ResolveTrackPlacementParams {
                tracks: updated_tracks.clone(),
                subject: PlacementSubject::TrackType(TrackType::of_track(track)),
                time_spans: Vec::new(),
                strategy: PlacementStrategy::AlwaysNew {
                    position: NewTrackPosition::Highest,
                },
            });

            let Some(placement_result @ PlacementResult::NewTrack { .. }) = placement_result
            else {
                continue;
            };

            let new_element_ids: Vec<String> = new_elements
                .iter()
                .map(|element| element.base().id.clone())
                .collect();

            let Some(applied) = apply_placement(&ApplyPlacementParams {
                tracks: updated_tracks.clone(),
                placement_result,
                elements: new_elements,
                new_track_id: Uuid::new_v4().to_string(),
                new_track_insert_index_override: None,
            }) else {
                continue;
            };

            updated_tracks = applied.updated_tracks;
            self.duplicated.extend(new_element_ids.into_iter().map(|element_id| {
                ElementRef {
                    track_id: applied.target_track_id.clone(),
                    element_id,
                }
            }));
        }

        scene.tracks = updated_tracks;

        if self.duplicated.is_empty() {
            return Ok(None);
        }
        Ok(Some(CommandResult::element_selection(
            self.duplicated.clone(),
        )))
    }

    fn undo(&mut self, project: &mut Project) {
        let Some(saved) = &self.saved_state else {
            return;
        };
        if let Some(scene) = active_scene_mut(project) {
            scene.tracks = saved.clone();
        }
    }
}
