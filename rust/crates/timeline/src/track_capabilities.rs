use scene::Track;

pub fn can_track_have_audio(track: &Track) -> bool {
    matches!(track, Track::Audio { .. } | Track::Video { .. })
}

pub fn can_track_be_hidden(track: &Track) -> bool {
    !matches!(track, Track::Audio { .. })
}
