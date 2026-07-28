use scene::{Element, SceneTracks, Track};

pub fn find_track_in_scene_tracks<'a>(tracks: &'a SceneTracks, track_id: &str) -> Option<&'a Track> {
    if tracks.main.id() == track_id {
        return Some(&tracks.main);
    }

    tracks
        .overlay
        .iter()
        .chain(tracks.audio.iter())
        .find(|track| track.id() == track_id)
}

pub fn update_track_in_scene_tracks(
    tracks: &SceneTracks,
    track_id: &str,
    update: impl Fn(&Track) -> Track,
) -> SceneTracks {
    if tracks.main.id() == track_id {
        return SceneTracks {
            main: update(&tracks.main),
            ..tracks.clone()
        };
    }

    if tracks.overlay.iter().any(|track| track.id() == track_id) {
        return SceneTracks {
            overlay: tracks
                .overlay
                .iter()
                .map(|track| {
                    if track.id() == track_id {
                        update(track)
                    } else {
                        track.clone()
                    }
                })
                .collect(),
            ..tracks.clone()
        };
    }

    if tracks.audio.iter().any(|track| track.id() == track_id) {
        return SceneTracks {
            audio: tracks
                .audio
                .iter()
                .map(|track| {
                    if track.id() == track_id {
                        update(track)
                    } else {
                        track.clone()
                    }
                })
                .collect(),
            ..tracks.clone()
        };
    }

    tracks.clone()
}

fn with_replaced_elements(track: &Track, elements: Vec<Element>) -> Track {
    let mut next = track.clone();
    *next.elements_mut() = elements;
    next
}

pub fn update_element_in_scene_tracks(
    tracks: &SceneTracks,
    track_id: &str,
    element_id: &str,
    update: impl Fn(&Element) -> Element,
    element_predicate: Option<&dyn Fn(&Element) -> bool>,
) -> SceneTracks {
    let update_in_track = |track: &Track| {
        let elements = track
            .elements()
            .iter()
            .map(|element| {
                if element.base().id != element_id {
                    return element.clone();
                }
                if let Some(predicate) = element_predicate
                    && !predicate(element)
                {
                    return element.clone();
                }
                update(element)
            })
            .collect();
        with_replaced_elements(track, elements)
    };

    update_track_in_scene_tracks(tracks, track_id, update_in_track)
}
