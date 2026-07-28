use std::path::PathBuf;

use audio::{
    AudibleClip, Error, apply_mastering, extract_rms_buckets, mix_range, render_export_buffer,
};
use media::Decoder;
use scene::RetimeConfig;
use time::MediaTime;

const TONE_WAV: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/tone.wav");

fn seconds(value: f64) -> MediaTime {
    MediaTime::from_seconds_f64(value).expect("finite seconds")
}

fn tone_clip(id: &str, start: f64, duration: f64) -> AudibleClip {
    AudibleClip {
        element_id: id.to_string(),
        path: PathBuf::from(TONE_WAV),
        start_time: seconds(start),
        duration: seconds(duration),
        trim_start: MediaTime::ZERO,
        trim_end: MediaTime::ZERO,
        volume_db: 0.0,
        retime: None,
        animations: None,
    }
}

fn decode_tone(start: f64, end: f64) -> (Vec<f32>, u32, usize) {
    let mut decoder = Decoder::open(TONE_WAV).expect("open tone.wav");
    let stream = decoder.info().audio_streams.first().cloned().expect("audio stream");
    let decoded = decoder
        .decode_audio_range(stream.index, seconds(start), seconds(end))
        .expect("decode tone");
    (decoded.samples, decoded.sample_rate, decoded.channels as usize)
}

#[test]
fn two_overlapping_tones_sum() {
    let clip_a = tone_clip("a", 0.0, 1.0);
    let clip_b = tone_clip("b", 0.0, 1.0);
    let single = mix_range(&[clip_a.clone()], (seconds(0.0), seconds(1.0)), 44100, 2).unwrap();
    let doubled = mix_range(&[clip_a, clip_b], (seconds(0.0), seconds(1.0)), 44100, 2).unwrap();
    assert_eq!(single.len(), doubled.len());
    for (one, two) in single.iter().zip(doubled.iter()) {
        assert!((two - 2.0 * one).abs() < 1e-6, "{two} != 2 * {one}");
    }
}

#[test]
fn volume_param_scales_samples() {
    let full = tone_clip("full", 0.0, 1.0);
    let mut half = tone_clip("half", 0.0, 1.0);
    half.volume_db = -6.020599913279624;
    let full_mix = mix_range(&[full], (seconds(0.0), seconds(1.0)), 44100, 2).unwrap();
    let half_mix = mix_range(&[half], (seconds(0.0), seconds(1.0)), 44100, 2).unwrap();
    for (f, h) in full_mix.iter().zip(half_mix.iter()) {
        assert!((h - f * 0.5).abs() < 1e-3, "{h} != 0.5 * {f}");
    }
}

#[test]
fn retime_2x_consumes_source_twice_as_fast() {
    let normal = tone_clip("normal", 0.0, 1.0);
    let mut fast = tone_clip("fast", 0.0, 0.5);
    fast.retime = Some(RetimeConfig {
        rate: 2.0,
        maintain_pitch: None,
    });
    let normal_mix = mix_range(&[normal], (seconds(0.0), seconds(1.0)), 44100, 2).unwrap();
    let fast_mix = mix_range(&[fast], (seconds(0.0), seconds(0.5)), 44100, 2).unwrap();
    let frames = fast_mix.len() / 2;
    for i in 0..frames {
        for channel in 0..2 {
            let expected = normal_mix[2 * i * 2 + channel];
            let actual = fast_mix[i * 2 + channel];
            assert!((actual - expected).abs() < 1e-6, "frame {i}: {actual} != {expected}");
        }
    }
}

#[test]
fn trim_start_offsets_source() {
    let mut clip = tone_clip("trimmed", 0.0, 0.5);
    clip.trim_start = seconds(0.5);
    let mix = mix_range(&[clip], (seconds(0.0), seconds(0.5)), 44100, 2).unwrap();
    let (decoded, _rate, channels) = decode_tone(0.5, 1.0);
    assert_eq!(channels, 1);
    let frames = mix.len() / 2;
    for i in 0..frames {
        for channel in 0..2 {
            let actual = mix[i * 2 + channel];
            assert!((actual - decoded[i]).abs() < 1e-6, "frame {i}: {actual} != {}", decoded[i]);
        }
    }
}

#[test]
fn rms_buckets_of_tone_match_hand_computed() {
    let (samples, rate, channels) = decode_tone(0.0, 1.0);
    let peak = samples.iter().fold(0.0_f32, |p, s| p.max(s.abs()));
    let expected_rms = peak / 2.0_f32.sqrt();
    let buckets = extract_rms_buckets(&samples, channels, rate, 10);
    assert_eq!(buckets.len(), 10);
    for bucket in &buckets {
        let relative = (bucket - expected_rms).abs() / expected_rms;
        assert!(relative < 0.05, "bucket {bucket} vs expected {expected_rms}");
    }
}

#[test]
fn mastering_limits_peaks_on_hot_input() {
    let frames = 4096;
    let mut samples = vec![1.5_f32; frames * 2];
    apply_mastering(&mut samples, 2, 44100);
    let peak = samples.iter().fold(0.0_f32, |p, s| p.max(s.abs()));
    assert!(peak <= 0.98, "peak was {peak}");
    assert!(peak > 0.9, "limiter over-attenuated: peak {peak}");
}

#[test]
fn mastering_leaves_quiet_input_untouched() {
    let mut samples: Vec<f32> = (0..1000).map(|i| (i as f32 / 1000.0) * 0.5).collect();
    let original = samples.clone();
    apply_mastering(&mut samples, 1, 44100);
    assert_eq!(samples, original);
}

#[test]
fn maintain_pitch_true_returns_typed_error() {
    let mut clip = tone_clip("pitched", 0.0, 0.5);
    clip.retime = Some(RetimeConfig {
        rate: 2.0,
        maintain_pitch: Some(true),
    });
    let result = mix_range(&[clip], (seconds(0.0), seconds(1.0)), 44100, 2);
    match result {
        Err(Error::MaintainPitchUnsupported { element_id, rate }) => {
            assert_eq!(element_id, "pitched");
            assert_eq!(rate, 2.0);
        }
        other => panic!("expected MaintainPitchUnsupported, got {other:?}"),
    }
}

#[test]
fn maintain_pitch_at_unity_rate_is_allowed() {
    let mut clip = tone_clip("unity", 0.0, 0.5);
    clip.retime = Some(RetimeConfig {
        rate: 1.0,
        maintain_pitch: Some(true),
    });
    assert!(mix_range(&[clip], (seconds(0.0), seconds(0.5)), 44100, 2).is_ok());
}

#[test]
fn render_export_buffer_mixes_and_masters() {
    let clip = tone_clip("a", 0.0, 1.0);
    let buffer = render_export_buffer(&[clip], seconds(1.0), 44100, 2).unwrap();
    assert_eq!(buffer.len(), 44100 * 2);
    let peak = buffer.iter().fold(0.0_f32, |p, s| p.max(s.abs()));
    assert!(peak > 0.05, "peak was {peak}");
    assert!(render_export_buffer(&[], seconds(1.0), 44100, 2).unwrap().is_empty());
}

#[test]
fn range_render_only_covers_requested_window() {
    let clip = tone_clip("a", 1.0, 1.0);
    let buffer = mix_range(&[clip], (seconds(0.0), seconds(2.0)), 44100, 2).unwrap();
    let first_half = &buffer[..44100 * 2];
    assert!(first_half.iter().all(|s| *s == 0.0));
    let second_peak = buffer[44100 * 2..].iter().fold(0.0_f32, |p, s| p.max(s.abs()));
    assert!(second_peak > 0.05);
}

#[test]
fn undecodable_clip_contributes_silence() {
    let mut clip = tone_clip("missing", 0.0, 1.0);
    clip.path = PathBuf::from("/nonexistent/nope.wav");
    let buffer = mix_range(&[clip], (seconds(0.0), seconds(0.5)), 44100, 2).unwrap();
    assert!(buffer.iter().all(|s| *s == 0.0));
}
