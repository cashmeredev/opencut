use scene::Bookmark;
use time::{FrameRate, MediaTime};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BookmarkPatch {
    pub note: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub duration: Option<Option<MediaTime>>,
}

pub fn find_bookmark_index(bookmarks: &[Bookmark], frame_time: MediaTime) -> Option<usize> {
    bookmarks.iter().position(|bookmark| bookmark.time == frame_time)
}

pub fn is_bookmark_at_time(bookmarks: &[Bookmark], frame_time: MediaTime) -> bool {
    find_bookmark_index(bookmarks, frame_time).is_some()
}

pub fn toggle_bookmark_in_array(
    bookmarks: &[Bookmark],
    frame_time: MediaTime,
) -> Vec<Bookmark> {
    if let Some(index) = find_bookmark_index(bookmarks, frame_time) {
        return bookmarks
            .iter()
            .enumerate()
            .filter(|(bookmark_index, _)| *bookmark_index != index)
            .map(|(_, bookmark)| bookmark.clone())
            .collect();
    }

    let mut next = bookmarks.to_vec();
    next.push(Bookmark {
        time: frame_time,
        note: None,
        color: None,
        duration: None,
    });
    next.sort_by_key(|bookmark| bookmark.time);
    next
}

pub fn remove_bookmark_from_array(
    bookmarks: &[Bookmark],
    frame_time: MediaTime,
) -> Vec<Bookmark> {
    bookmarks
        .iter()
        .filter(|bookmark| bookmark.time != frame_time)
        .cloned()
        .collect()
}

pub fn update_bookmark_in_array(
    bookmarks: &[Bookmark],
    frame_time: MediaTime,
    updates: &BookmarkPatch,
) -> Vec<Bookmark> {
    let Some(index) = find_bookmark_index(bookmarks, frame_time) else {
        return bookmarks.to_vec();
    };

    let mut next = bookmarks.to_vec();
    let bookmark = &mut next[index];
    if let Some(note) = &updates.note {
        bookmark.note = note.clone();
    }
    if let Some(color) = &updates.color {
        bookmark.color = color.clone();
    }
    if let Some(duration) = &updates.duration {
        bookmark.duration = *duration;
    }
    next
}

pub fn move_bookmark_in_array(
    bookmarks: &[Bookmark],
    from_time: MediaTime,
    to_time: MediaTime,
) -> Vec<Bookmark> {
    let Some(index) = find_bookmark_index(bookmarks, from_time) else {
        return bookmarks.to_vec();
    };

    let mut next = bookmarks.to_vec();
    next[index].time = to_time;
    next.sort_by_key(|bookmark| bookmark.time);
    next
}

pub fn get_frame_time(time: MediaTime, fps: FrameRate) -> MediaTime {
    time.round_to_frame(fps).unwrap_or(time)
}
