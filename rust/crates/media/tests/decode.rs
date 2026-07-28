use media::{Decoder, Error, MediaTime, thumbnail, waveform_summary};

const SAMPLE_MP4: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sample.mp4");
const TONE_WAV: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/tone.wav");

fn seconds(value: f64) -> MediaTime {
    MediaTime::from_seconds_f64(value).expect("finite seconds")
}

#[test]
fn reports_media_info() {
    let decoder = Decoder::open(SAMPLE_MP4).expect("open sample.mp4");
    let info = decoder.info();
    let duration_seconds = info.duration.to_seconds_f64();
    assert!(
        (duration_seconds - 2.0).abs() < 0.05,
        "duration was {duration_seconds}"
    );
    let video = info.video_streams.first().expect("video stream");
    assert_eq!(video.width, 320);
    assert_eq!(video.height, 180);
    assert!(
        (video.frame_rate - 30.0).abs() < 0.01,
        "frame rate was {}",
        video.frame_rate
    );
    assert_eq!(video.frame_count, Some(60));
    let audio = info.audio_streams.first().expect("audio stream");
    assert_eq!(audio.sample_rate, 44100);
    assert!(audio.channels >= 1);
}

#[test]
fn decodes_video_frame_at_zero_and_one_second() {
    let mut decoder = Decoder::open(SAMPLE_MP4).expect("open sample.mp4");
    let first = decoder
        .decode_video_frame(0, seconds(0.0))
        .expect("frame at t=0");
    assert_eq!(first.width, 320);
    assert_eq!(first.height, 180);
    assert_eq!(first.data.len(), 320 * 180 * 4);
    let first_pixel = &first.data[0..4];
    assert!(
        first.data.chunks_exact(4).any(|pixel| pixel != first_pixel),
        "frame at t=0 is uniform"
    );
    let later = decoder
        .decode_video_frame(0, seconds(1.0))
        .expect("frame at t=1");
    assert_eq!(later.width, 320);
    assert_eq!(later.height, 180);
    assert!(
        later.data.chunks_exact(4).any(|pixel| pixel != first_pixel),
        "frame at t=1 is uniform"
    );
    assert_ne!(first.data, later.data, "frames at t=0 and t=1 are identical");
}

#[test]
fn decodes_sequential_frames_forward() {
    let mut decoder = Decoder::open(SAMPLE_MP4).expect("open sample.mp4");
    let mut previous = decoder.decode_video_frame(0, seconds(0.25)).expect("t=0.25");
    for step in 1..8 {
        let time = 0.25 + f64::from(step) * 0.2;
        let frame = decoder.decode_video_frame(0, seconds(time)).expect("frame");
        assert_eq!((frame.width, frame.height), (320, 180));
        previous = frame;
    }
    let _ = previous;
}

#[test]
fn time_past_eof_returns_last_frame() {
    let mut decoder = Decoder::open(SAMPLE_MP4).expect("open sample.mp4");
    let frame = decoder
        .decode_video_frame(0, seconds(10.0))
        .expect("frame past EOF");
    assert_eq!(frame.width, 320);
    assert_eq!(frame.height, 180);
    assert_eq!(frame.data.len(), 320 * 180 * 4);
}

#[test]
fn rejects_wrong_stream_type() {
    let mut decoder = Decoder::open(SAMPLE_MP4).expect("open sample.mp4");
    let info = decoder.info();
    let audio_index = info.audio_streams.first().expect("audio stream").index;
    let result = decoder.decode_video_frame(audio_index, seconds(0.0));
    assert!(matches!(result, Err(Error::NotAVideoStream(_))));
    let video_index = info.video_streams.first().expect("video stream").index;
    let result = decoder.decode_audio_range(video_index, seconds(0.0), seconds(1.0));
    assert!(matches!(result, Err(Error::NotAnAudioStream(_))));
    let result = decoder.decode_video_frame(99, seconds(0.0));
    assert!(matches!(result, Err(Error::StreamNotFound(99))));
}

#[test]
fn decodes_tone_waveform() {
    let mut decoder = Decoder::open(TONE_WAV).expect("open tone.wav");
    let info = decoder.info();
    let audio = info.audio_streams.first().expect("audio stream");
    assert_eq!(audio.sample_rate, 44100);
    let decoded = decoder
        .decode_audio_range(audio.index, seconds(0.0), seconds(1.0))
        .expect("decode range");
    assert_eq!(decoded.sample_rate, 44100);
    let channels = decoded.channels as usize;
    let frame_count = decoded.samples.len() / channels;
    assert!(
        (frame_count as i64 - 44100).abs() <= 2,
        "frame count was {frame_count}"
    );
    let mean_square: f32 =
        decoded.samples.iter().map(|sample| sample * sample).sum::<f32>()
            / decoded.samples.len() as f32;
    assert!(mean_square.sqrt() > 0.05, "rms was {}", mean_square.sqrt());
    assert!(
        decoded.samples.iter().all(|sample| sample.abs() <= 1.0),
        "samples exceed [-1, 1]"
    );
    let mut crossings = 0usize;
    let mut previous_sign = decoded.samples[0].is_sign_negative();
    for &sample in &decoded.samples[1..] {
        let sign = sample.is_sign_negative();
        if sign != previous_sign {
            crossings += 1;
            previous_sign = sign;
        }
    }
    assert!(
        (crossings as i64 - 440).abs() <= 22,
        "zero crossings were {crossings}"
    );
}

#[test]
fn decodes_aac_audio_range() {
    let mut decoder = Decoder::open(SAMPLE_MP4).expect("open sample.mp4");
    let info = decoder.info();
    let audio = info.audio_streams.first().expect("audio stream");
    let decoded = decoder
        .decode_audio_range(audio.index, seconds(0.5), seconds(1.5))
        .expect("decode aac range");
    assert_eq!(decoded.sample_rate, 44100);
    let channels = decoded.channels as usize;
    let frame_count = decoded.samples.len() / channels;
    assert!(
        (frame_count as i64 - 44100).abs() <= 100,
        "frame count was {frame_count}"
    );
    let mean_square: f32 =
        decoded.samples.iter().map(|sample| sample * sample).sum::<f32>()
            / decoded.samples.len() as f32;
    assert!(mean_square.sqrt() > 0.01, "rms was {}", mean_square.sqrt());
}

#[test]
fn rejects_invalid_audio_range() {
    let mut decoder = Decoder::open(TONE_WAV).expect("open tone.wav");
    let result = decoder.decode_audio_range(0, seconds(1.0), seconds(0.5));
    assert!(matches!(result, Err(Error::InvalidTimeRange)));
}

#[test]
fn scales_thumbnail_to_max_dimension() {
    let thumb = thumbnail(SAMPLE_MP4, 64).expect("thumbnail");
    assert_eq!(thumb.width, 64);
    assert_eq!(thumb.height, 36);
    assert_eq!(thumb.data.len(), 64 * 36 * 4);
    let first_pixel = &thumb.data[0..4];
    assert!(
        thumb.data.chunks_exact(4).any(|pixel| pixel != first_pixel),
        "thumbnail is uniform"
    );
}

#[test]
fn summarizes_waveform_buckets() {
    let buckets = waveform_summary(TONE_WAV, 10).expect("waveform summary");
    assert_eq!(buckets.len(), 10);
    for (low, high) in &buckets {
        assert!(low <= high, "bucket min {low} exceeds max {high}");
        assert!(*low >= -1.0 && *high <= 1.0, "bucket ({low}, {high}) out of range");
    }
    assert!(
        buckets.iter().any(|(low, high)| *high > 0.1 && *low < -0.1),
        "no bucket carries the 220Hz tone"
    );
    let single = waveform_summary(TONE_WAV, 1).expect("single bucket");
    assert_eq!(single.len(), 1);
    assert!(single[0].1 > 0.1);
    assert!(matches!(waveform_summary(TONE_WAV, 0), Err(Error::InvalidBucketCount)));
}
