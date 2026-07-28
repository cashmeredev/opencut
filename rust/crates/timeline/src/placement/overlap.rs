use scene::Element;
use time::MediaTime;

use super::PlacementTimeSpan;

fn would_element_overlap(
    elements: &[Element],
    start_time: MediaTime,
    end_time: MediaTime,
    exclude_element_id: Option<&str>,
) -> bool {
    elements.iter().any(|element| {
        if exclude_element_id == Some(element.base().id.as_str()) {
            return false;
        }

        let element_end = element.base().start_time + element.base().duration;
        start_time < element_end && end_time > element.base().start_time
    })
}

pub fn can_place_time_spans_on_track(
    elements: &[Element],
    time_spans: &[PlacementTimeSpan],
) -> bool {
    time_spans.iter().all(|span| {
        !would_element_overlap(
            elements,
            span.start_time,
            span.start_time + span.duration,
            span.exclude_element_id.as_deref(),
        )
    })
}
