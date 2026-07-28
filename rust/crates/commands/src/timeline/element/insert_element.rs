use scene::{AudioElement, Element, ElementRef, ParamValue, Project, SceneTracks};
use timeline::placement::apply::ApplyPlacementParams;
use timeline::placement::{
    PlacementResult, PlacementStrategy, PlacementSubject, PlacementTimeSpan,
    ResolveTrackPlacementParams, apply_placement, can_element_go_on_track,
    resolve_track_placement,
};
use timeline::{ElementType, TrackType};
use uuid::Uuid;

use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;

#[derive(Clone, Debug, PartialEq)]
pub enum InsertElementPlacement {
    Explicit {
        track_id: String,
    },
    Auto {
        track_type: Option<TrackType>,
        insert_index: Option<usize>,
    },
}

fn requires_media_id(element: &Element) -> bool {
    matches!(
        element,
        Element::Video(_) | Element::Image(_) | Element::Audio(AudioElement::Upload(_))
    )
}

fn has_media_id_value(element: &Element) -> bool {
    match element {
        Element::Video(video) => !video.media_id.is_empty(),
        Element::Image(image) => !image.media_id.is_empty(),
        Element::Audio(AudioElement::Upload(upload)) => !upload.media_id.is_empty(),
        _ => false,
    }
}

fn validate_element_basics(element: &Element) -> bool {
    if requires_media_id(element) && !has_media_id_value(element) {
        return false;
    }

    match element {
        Element::Audio(AudioElement::Library(library)) => !library.source_url.is_empty(),
        Element::Sticker(sticker) => !sticker.sticker_id.is_empty(),
        Element::Graphic(graphic) => !graphic.definition_id.is_empty(),
        Element::Text(text) => matches!(
            text.base.params.get("content"),
            Some(ParamValue::String(content)) if !content.is_empty()
        ),
        Element::Effect(effect) => !effect.effect_type.is_empty(),
        _ => true,
    }
}

pub struct InsertElementCommand {
    element: Element,
    placement: InsertElementPlacement,
    element_id: String,
    target_track_id: Option<String>,
    saved_state: Option<SceneTracks>,
}

impl InsertElementCommand {
    pub fn new(element: Element, placement: InsertElementPlacement) -> Self {
        Self {
            element,
            placement,
            element_id: Uuid::new_v4().to_string(),
            target_track_id: None,
            saved_state: None,
        }
    }

    pub fn element_id(&self) -> &str {
        &self.element_id
    }

    pub fn track_id(&self) -> Option<&str> {
        self.target_track_id.as_deref()
    }

    fn apply_placement_result(
        &self,
        tracks: &SceneTracks,
        element: &Element,
    ) -> Option<(SceneTracks, String)> {
        let element_type = ElementType::of_element(element);

        if let InsertElementPlacement::Auto {
            track_type: Some(track_type),
            ..
        } = &self.placement
            && !can_element_go_on_track(element_type, *track_type)
        {
            return None;
        }

        let subject = match &self.placement {
            InsertElementPlacement::Auto {
                track_type: Some(track_type),
                ..
            } => PlacementSubject::TrackType(*track_type),
            _ => PlacementSubject::ElementType(element_type),
        };

        let strategy = match &self.placement {
            InsertElementPlacement::Explicit { track_id } => PlacementStrategy::Explicit {
                track_id: track_id.clone(),
            },
            InsertElementPlacement::Auto { .. } => PlacementStrategy::FirstAvailable,
        };

        let placement_result = resolve_track_placement(&ResolveTrackPlacementParams {
            tracks: tracks.clone(),
            subject,
            time_spans: vec![PlacementTimeSpan {
                start_time: element.base().start_time,
                duration: element.base().duration,
                exclude_element_id: None,
            }],
            strategy,
        })?;

        let mut element_to_place = element.clone();
        if let PlacementResult::ExistingTrack {
            adjusted_start_time: Some(adjusted),
            ..
        } = &placement_result
        {
            element_to_place.base_mut().start_time = *adjusted;
        }

        let insert_index_override = match &self.placement {
            InsertElementPlacement::Auto { insert_index, .. } => *insert_index,
            InsertElementPlacement::Explicit { .. } => None,
        };

        let applied = apply_placement(&ApplyPlacementParams {
            tracks: tracks.clone(),
            placement_result,
            elements: vec![element_to_place],
            new_track_id: Uuid::new_v4().to_string(),
            new_track_insert_index_override: insert_index_override,
        })?;

        Some((applied.updated_tracks, applied.target_track_id))
    }
}

impl Command for InsertElementCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let Some(scene) = active_scene_mut(project) else {
            return Ok(None);
        };

        self.saved_state = Some(scene.tracks.clone());

        if !validate_element_basics(&self.element) {
            return Ok(None);
        }

        let mut new_element = self.element.clone();
        new_element.base_mut().id = self.element_id.clone();

        let Some((updated_tracks, target_track_id)) =
            self.apply_placement_result(&scene.tracks, &new_element)
        else {
            return Ok(None);
        };

        self.target_track_id = Some(target_track_id.clone());
        scene.tracks = updated_tracks;

        Ok(Some(CommandResult::element_selection(vec![ElementRef {
            track_id: target_track_id,
            element_id: self.element_id.clone(),
        }])))
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
