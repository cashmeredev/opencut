use std::cell::RefCell;
use std::rc::Rc;

use scene::{AudioElement, Element, ElementRef, Project, RetimeConfig, SceneTracks};
use time::MediaTime;
use uuid::Uuid;

use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;
use crate::tracks::all_tracks_mut;

const MIN_RETIME_RATE: f64 = 0.01;
const MAX_RETIME_RATE: f64 = 5.0;
const DEFAULT_RETIME_RATE: f64 = 1.0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RetainSide {
    #[default]
    Both,
    Left,
    Right,
}

fn clamp_retime_rate(rate: f64) -> f64 {
    if !rate.is_finite() || rate <= 0.0 {
        return DEFAULT_RETIME_RATE;
    }
    rate.clamp(MIN_RETIME_RATE, MAX_RETIME_RATE)
}

fn source_span_at_clip_time(clip_time: MediaTime, retime: Option<RetimeConfig>) -> f64 {
    let rate = clamp_retime_rate(retime.map(|config| config.rate).unwrap_or(1.0));
    (clip_time.as_ticks() as f64 * rate).max(0.0)
}

fn round_to_ticks(time: f64) -> MediaTime {
    MediaTime::from_ticks(time.round() as i64)
}

fn retime_of(element: &Element) -> Option<RetimeConfig> {
    match element {
        Element::Video(video) => video.retime,
        Element::Audio(AudioElement::Upload(upload)) => upload.retime,
        Element::Audio(AudioElement::Library(library)) => library.retime,
        _ => None,
    }
}

pub struct SplitElementsCommand {
    elements: Vec<ElementRef>,
    split_time: MediaTime,
    retain_side: RetainSide,
    saved_state: Option<SceneTracks>,
    right_side: Rc<RefCell<Vec<ElementRef>>>,
}

impl SplitElementsCommand {
    pub fn new(elements: Vec<ElementRef>, split_time: MediaTime, retain_side: RetainSide) -> Self {
        Self {
            elements,
            split_time,
            retain_side,
            saved_state: None,
            right_side: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn right_side_handle(&self) -> Rc<RefCell<Vec<ElementRef>>> {
        Rc::clone(&self.right_side)
    }

    pub fn right_side_elements(&self) -> Vec<ElementRef> {
        self.right_side.borrow().clone()
    }

    fn split_element(
        &self,
        track_id: &str,
        element: Element,
        right_side: &mut Vec<ElementRef>,
    ) -> Vec<Element> {
        let start = element.base().start_time;
        let end = start + element.base().duration;

        if self.split_time <= start || self.split_time >= end {
            return vec![element];
        }

        let relative_time = self.split_time - start;
        let left_visible_duration = relative_time;
        let right_visible_duration = element.base().duration - relative_time;
        let retime = retime_of(&element);

        let left_source_span = round_to_ticks(source_span_at_clip_time(
            left_visible_duration,
            retime,
        ));
        let total_source_span =
            round_to_ticks(source_span_at_clip_time(element.base().duration, retime));
        let right_source_span = total_source_span - left_source_span;

        let (left_animations, right_animations) = animation::split_animations_at_time_with_options(
            element.base().animations.as_ref(),
            relative_time,
            true,
        );

        let left_trim_end = element.base().trim_end + right_source_span;
        let right_trim_start = element.base().trim_start + left_source_span;

        match self.retain_side {
            RetainSide::Left => {
                let mut left = element;
                {
                    let base = left.base_mut();
                    base.duration = left_visible_duration;
                    base.trim_end = left_trim_end;
                    base.name = format!("{} (left)", base.name);
                    base.animations = left_animations;
                }
                vec![left]
            }
            RetainSide::Right => {
                let mut right = element;
                let new_id = Uuid::new_v4().to_string();
                right_side.push(ElementRef {
                    track_id: track_id.to_string(),
                    element_id: new_id.clone(),
                });
                {
                    let base = right.base_mut();
                    base.id = new_id;
                    base.start_time = self.split_time;
                    base.duration = right_visible_duration;
                    base.trim_start = right_trim_start;
                    base.name = format!("{} (right)", base.name);
                    base.animations = right_animations;
                }
                vec![right]
            }
            RetainSide::Both => {
                let mut left = element;
                let mut right = left.clone();
                let right_id = Uuid::new_v4().to_string();
                right_side.push(ElementRef {
                    track_id: track_id.to_string(),
                    element_id: right_id.clone(),
                });
                {
                    let base = left.base_mut();
                    base.duration = left_visible_duration;
                    base.trim_end = left_trim_end;
                    base.name = format!("{} (left)", base.name);
                    base.animations = left_animations;
                }
                {
                    let base = right.base_mut();
                    base.id = right_id;
                    base.start_time = self.split_time;
                    base.duration = right_visible_duration;
                    base.trim_start = right_trim_start;
                    base.name = format!("{} (right)", base.name);
                    base.animations = right_animations;
                }
                vec![left, right]
            }
        }
    }
}

impl Command for SplitElementsCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let Some(scene) = active_scene_mut(project) else {
            return Ok(None);
        };

        self.saved_state = Some(scene.tracks.clone());
        self.right_side.borrow_mut().clear();

        for track in all_tracks_mut(&mut scene.tracks) {
            let targets: Vec<&str> = self
                .elements
                .iter()
                .filter(|target| target.track_id == track.id())
                .map(|target| target.element_id.as_str())
                .collect();
            if targets.is_empty() {
                continue;
            }

            let track_id = track.id().to_string();
            let previous = std::mem::take(track.elements_mut());
            let mut next = Vec::with_capacity(previous.len());
            {
                let mut right_side = self.right_side.borrow_mut();
                for element in previous {
                    if !targets.contains(&element.base().id.as_str()) {
                        next.push(element);
                        continue;
                    }
                    next.extend(self.split_element(&track_id, element, &mut right_side));
                }
            }
            *track.elements_mut() = next;
        }

        let right_side = self.right_side.borrow();
        if right_side.is_empty() {
            return Ok(None);
        }
        Ok(Some(CommandResult::element_selection(right_side.clone())))
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
