use time::{FrameRate, MediaTime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaybackEvent {
    Started,
    Paused,
    Seeked(MediaTime),
    Updated(MediaTime),
    ReachedEnd(MediaTime),
}

pub struct PlaybackClock {
    is_playing: bool,
    current_time: MediaTime,
    total_duration: MediaTime,
    frame_rate: Option<FrameRate>,
    anchor_media: MediaTime,
    anchor_wall_seconds: f64,
    volume: f64,
    previous_volume: f64,
    muted: bool,
    is_scrubbing: bool,
}

impl Default for PlaybackClock {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaybackClock {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            current_time: MediaTime::ZERO,
            total_duration: MediaTime::ZERO,
            frame_rate: None,
            anchor_media: MediaTime::ZERO,
            anchor_wall_seconds: 0.0,
            volume: 1.0,
            previous_volume: 1.0,
            muted: false,
            is_scrubbing: false,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn current_time(&self) -> MediaTime {
        self.current_time
    }

    pub fn total_duration(&self) -> MediaTime {
        self.total_duration
    }

    pub fn volume(&self) -> f64 {
        self.volume
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn is_scrubbing(&self) -> bool {
        self.is_scrubbing
    }

    pub fn set_total_duration(&mut self, duration: MediaTime) -> Vec<PlaybackEvent> {
        self.total_duration = duration;
        let clamped = self.clamp(self.current_time);
        let reached_end = self.is_playing && clamped >= self.total_duration;
        if clamped == self.current_time && !reached_end {
            return Vec::new();
        }
        self.current_time = clamped;
        let mut events = vec![PlaybackEvent::Seeked(clamped)];
        if reached_end {
            self.is_playing = false;
            events.push(PlaybackEvent::Paused);
        }
        events
    }

    pub fn set_frame_rate(&mut self, frame_rate: Option<FrameRate>) {
        self.frame_rate = frame_rate;
    }

    pub fn play(&mut self, wall_seconds: f64) -> Vec<PlaybackEvent> {
        if self.total_duration <= MediaTime::ZERO {
            return Vec::new();
        }
        let mut events = Vec::new();
        if self.current_time >= self.total_duration {
            self.current_time = MediaTime::ZERO;
            events.push(PlaybackEvent::Seeked(MediaTime::ZERO));
        }
        self.is_playing = true;
        self.anchor_media = self.current_time;
        self.anchor_wall_seconds = wall_seconds;
        events.push(PlaybackEvent::Started);
        events
    }

    pub fn pause(&mut self) -> Vec<PlaybackEvent> {
        if !self.is_playing {
            return Vec::new();
        }
        self.is_playing = false;
        vec![PlaybackEvent::Paused]
    }

    pub fn toggle(&mut self, wall_seconds: f64) -> Vec<PlaybackEvent> {
        if self.is_playing {
            self.pause()
        } else {
            self.play(wall_seconds)
        }
    }

    pub fn seek(&mut self, time: MediaTime, wall_seconds: f64) -> Vec<PlaybackEvent> {
        self.current_time = self.clamp(time);
        if self.is_playing {
            self.anchor_media = self.current_time;
            self.anchor_wall_seconds = wall_seconds;
        }
        vec![PlaybackEvent::Seeked(self.current_time)]
    }

    pub fn tick(&mut self, wall_seconds: f64) -> Vec<PlaybackEvent> {
        if !self.is_playing {
            return Vec::new();
        }
        let elapsed = wall_seconds - self.anchor_wall_seconds;
        let Some(elapsed_time) = MediaTime::from_seconds_f64(elapsed) else {
            return Vec::new();
        };
        let raw = MediaTime::from_ticks(
            self.anchor_media.as_ticks().saturating_add(elapsed_time.as_ticks()),
        );
        let rounded = self.round_to_frame(raw);

        if rounded >= self.total_duration {
            self.is_playing = false;
            self.current_time = self.total_duration;
            return vec![
                PlaybackEvent::Paused,
                PlaybackEvent::ReachedEnd(self.total_duration),
            ];
        }
        if rounded == self.current_time {
            return Vec::new();
        }
        self.current_time = rounded;
        vec![PlaybackEvent::Updated(rounded)]
    }

    pub fn set_volume(&mut self, volume: f64) {
        let clamped = volume.clamp(0.0, 1.0);
        self.volume = clamped;
        self.muted = clamped == 0.0;
        if clamped > 0.0 {
            self.previous_volume = clamped;
        }
    }

    pub fn mute(&mut self) {
        if self.volume > 0.0 {
            self.previous_volume = self.volume;
        }
        self.muted = true;
        self.volume = 0.0;
    }

    pub fn unmute(&mut self) {
        self.muted = false;
        self.volume = self.previous_volume;
    }

    pub fn toggle_mute(&mut self) {
        if self.muted {
            self.unmute();
        } else {
            self.mute();
        }
    }

    pub fn set_scrubbing(&mut self, is_scrubbing: bool) {
        self.is_scrubbing = is_scrubbing;
    }

    fn clamp(&self, time: MediaTime) -> MediaTime {
        time.clamp(MediaTime::ZERO, self.total_duration)
    }

    fn round_to_frame(&self, time: MediaTime) -> MediaTime {
        let Some(rate) = self.frame_rate else {
            return time;
        };
        let Some(frame) = time.to_frame_round(rate) else {
            return time;
        };
        MediaTime::from_frame(frame, rate).unwrap_or(time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FPS_30: FrameRate = FrameRate {
        numerator: 30,
        denominator: 1,
    };

    fn clock() -> PlaybackClock {
        let mut clock = PlaybackClock::new();
        clock.set_frame_rate(Some(FPS_30));
        clock.set_total_duration(MediaTime::from_seconds_f64(4.0).unwrap());
        clock
    }

    #[test]
    fn play_at_end_restarts_from_zero() {
        let mut clock = clock();
        clock.seek(MediaTime::from_seconds_f64(10.0).unwrap(), 0.0);
        let events = clock.play(0.0);
        assert!(events.contains(&PlaybackEvent::Seeked(MediaTime::ZERO)));
        assert!(events.contains(&PlaybackEvent::Started));
    }

    #[test]
    fn play_with_empty_timeline_does_nothing() {
        let mut clock = PlaybackClock::new();
        assert!(clock.play(0.0).is_empty());
        assert!(!clock.is_playing());
    }

    #[test]
    fn tick_rounds_to_frames() {
        let mut clock = clock();
        clock.play(0.0);
        let events = clock.tick(0.5);
        let expected = MediaTime::from_frame(15, FPS_30).unwrap();
        assert_eq!(events, vec![PlaybackEvent::Updated(expected)]);
        assert_eq!(clock.current_time(), expected);
    }

    #[test]
    fn tick_stops_at_end() {
        let mut clock = clock();
        clock.play(0.0);
        let events = clock.tick(10.0);
        assert!(events.contains(&PlaybackEvent::Paused));
        assert_eq!(clock.current_time(), clock.total_duration());
        assert!(!clock.is_playing());
    }

    #[test]
    fn tick_same_frame_emits_nothing() {
        let mut clock = clock();
        clock.play(0.0);
        assert!(clock.tick(0.001).is_empty());
    }

    #[test]
    fn seek_clamps_into_timeline() {
        let mut clock = clock();
        clock.seek(MediaTime::from_seconds_f64(99.0).unwrap(), 0.0);
        assert_eq!(clock.current_time(), clock.total_duration());
    }

    #[test]
    fn seek_while_playing_reanchors() {
        let mut clock = clock();
        clock.play(0.0);
        clock.seek(MediaTime::from_seconds_f64(1.0).unwrap(), 5.0);
        let events = clock.tick(6.0);
        let expected = MediaTime::from_frame(60, FPS_30).unwrap();
        assert_eq!(events, vec![PlaybackEvent::Updated(expected)]);
    }

    #[test]
    fn volume_mute_cycle_restores_previous() {
        let mut clock = clock();
        clock.set_volume(0.7);
        clock.mute();
        assert_eq!(clock.volume(), 0.0);
        assert!(clock.is_muted());
        clock.unmute();
        assert_eq!(clock.volume(), 0.7);
        assert!(!clock.is_muted());
    }

    #[test]
    fn zero_volume_counts_as_muted() {
        let mut clock = clock();
        clock.set_volume(0.5);
        clock.set_volume(0.0);
        assert!(clock.is_muted());
        clock.unmute();
        assert_eq!(clock.volume(), 0.5);
    }

    #[test]
    fn shrinking_duration_pauses_at_end() {
        let mut clock = clock();
        clock.play(0.0);
        clock.tick(2.0);
        let events = clock.set_total_duration(MediaTime::from_seconds_f64(1.0).unwrap());
        assert!(events.contains(&PlaybackEvent::Paused));
        assert_eq!(clock.current_time(), MediaTime::from_seconds_f64(1.0).unwrap());
    }
}
