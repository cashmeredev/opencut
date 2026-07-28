use scene::{Element, SceneTracks, Track};
use time::MediaTime;

pub const MAIN_TRACK_NAME: &str = "Main Track";

pub fn get_earliest_main_track_element<'a>(
    main_track: &'a Track,
    exclude_element_id: Option<&str>,
) -> Option<&'a Element> {
    main_track
        .elements()
        .iter()
        .filter(|element| exclude_element_id != Some(element.base().id.as_str()))
        .reduce(|earliest, element| {
            if element.base().start_time < earliest.base().start_time {
                element
            } else {
                earliest
            }
        })
}

pub fn enforce_main_track_start(
    tracks: &SceneTracks,
    target_track_id: &str,
    requested_start_time: MediaTime,
    exclude_element_id: Option<&str>,
) -> MediaTime {
    if tracks.main.id() != target_track_id {
        return requested_start_time;
    }

    let Some(earliest_element) =
        get_earliest_main_track_element(&tracks.main, exclude_element_id)
    else {
        return MediaTime::ZERO;
    };

    if requested_start_time <= earliest_element.base().start_time {
        return MediaTime::ZERO;
    }

    requested_start_time
}
