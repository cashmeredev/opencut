use scene::{SceneTracks, Track};

pub fn find_track<'a>(tracks: &'a SceneTracks, track_id: &str) -> Option<&'a Track> {
    if tracks.main.id() == track_id {
        return Some(&tracks.main);
    }
    tracks
        .overlay
        .iter()
        .find(|track| track.id() == track_id)
        .or_else(|| tracks.audio.iter().find(|track| track.id() == track_id))
}

pub fn find_track_mut<'a>(
    tracks: &'a mut SceneTracks,
    track_id: &str,
) -> Option<&'a mut Track> {
    if tracks.main.id() == track_id {
        return Some(&mut tracks.main);
    }
    tracks
        .overlay
        .iter_mut()
        .find(|track| track.id() == track_id)
        .or_else(|| tracks.audio.iter_mut().find(|track| track.id() == track_id))
}

pub fn all_tracks(tracks: &SceneTracks) -> impl Iterator<Item = &Track> {
    tracks
        .overlay
        .iter()
        .chain(std::iter::once(&tracks.main))
        .chain(tracks.audio.iter())
}

pub fn all_tracks_mut(tracks: &mut SceneTracks) -> impl Iterator<Item = &mut Track> {
    tracks
        .overlay
        .iter_mut()
        .chain(std::iter::once(&mut tracks.main))
        .chain(tracks.audio.iter_mut())
}
