use media::{AudioSamples, Decoder};
use time::MediaTime;

use crate::Error;
use crate::clips::AudibleClip;

pub const MASTER_LIMITER_THRESHOLD_DB: f64 = -1.0;
pub const MASTER_LIMITER_RATIO: f64 = 20.0;
pub const MASTER_LIMITER_ATTACK_SECONDS: f64 = 0.001;
pub const MASTER_LIMITER_RELEASE_SECONDS: f64 = 0.12;
pub const MASTER_OUTPUT_HEADROOM: f64 = 0.98;
pub const MASTER_OUTPUT_HEADROOM_F32: f32 = 0.98;

fn decode_clip_source(clip: &AudibleClip, source_start: f64, source_end: f64) -> Option<AudioSamples> {
    let start = MediaTime::from_seconds_f64(source_start.max(0.0))?;
    let end = MediaTime::from_seconds_f64(source_end.max(0.0))?;
    if end <= start {
        return None;
    }
    let mut decoder = Decoder::open(&clip.path).ok()?;
    let stream = decoder.info().audio_streams.first().cloned()?;
    decoder.decode_audio_range(stream.index, start, end).ok()
}

fn mix_clip(
    clip: &AudibleClip,
    range_start_seconds: f64,
    out_frames: usize,
    sample_rate: u32,
    channel_count: usize,
    output: &mut [f32],
) {
    let clip_start_seconds = clip.start_time.to_seconds_f64();
    let clip_end_seconds = clip_start_seconds + clip.duration.to_seconds_f64();
    let rate = clip.effective_rate();
    let trim_start_seconds = clip.trim_start.to_seconds_f64();
    let rate_f64 = f64::from(sample_rate);

    let first_frame = ((clip_start_seconds - range_start_seconds) * rate_f64)
        .ceil()
        .max(0.0) as usize;
    let last_frame = (((clip_end_seconds - range_start_seconds) * rate_f64)
        .ceil()
        .max(0.0) as usize)
        .min(out_frames);
    if last_frame <= first_frame {
        return;
    }

    let source_start = trim_start_seconds
        + (first_frame as f64 / rate_f64 + range_start_seconds - clip_start_seconds) * rate;
    let source_end = trim_start_seconds
        + (last_frame as f64 / rate_f64 + range_start_seconds - clip_start_seconds) * rate;
    let Some(decoded) = decode_clip_source(clip, source_start, source_end) else {
        return;
    };
    let decoded_channels = decoded.channels as usize;
    let decoded_frames = decoded.samples.len() / decoded_channels;
    let decoded_rate = f64::from(decoded.sample_rate);
    let static_gain = clip.static_gain() as f32;
    let animated_volume = clip.has_animated_volume();

    for frame in first_frame..last_frame {
        let clip_time = frame as f64 / rate_f64 + range_start_seconds - clip_start_seconds;
        let source_time = trim_start_seconds + clip_time * rate;
        let source_position = (source_time - source_start) * decoded_rate;
        if source_position >= decoded_frames as f64 {
            break;
        }
        let source_position = source_position.max(0.0);
        let lower = source_position.floor() as usize;
        let upper = (lower + 1).min(decoded_frames.saturating_sub(1));
        let fraction = (source_position - lower as f64) as f32;
        let gain = if animated_volume {
            clip.gain_at(MediaTime::from_seconds_f64(clip_time).unwrap_or(MediaTime::ZERO)) as f32
        } else {
            static_gain
        };
        for channel in 0..channel_count {
            let source_channel = channel.min(decoded_channels - 1);
            let a = decoded.samples[lower * decoded_channels + source_channel];
            let b = decoded.samples[upper * decoded_channels + source_channel];
            output[frame * channel_count + channel] +=
                (a * (1.0 - fraction) + b * fraction) * gain;
        }
    }
}

pub fn mix_range(
    clips: &[AudibleClip],
    range: (MediaTime, MediaTime),
    sample_rate: u32,
    channels: u32,
) -> Result<Vec<f32>, Error> {
    if range.1 <= range.0 {
        return Err(Error::InvalidTimeRange);
    }
    if sample_rate == 0 {
        return Err(Error::InvalidSampleRate);
    }
    if channels == 0 {
        return Err(Error::InvalidChannelCount);
    }
    for clip in clips {
        if clip.requires_maintain_pitch()
            && clip.end_time() > range.0
            && clip.start_time < range.1
        {
            return Err(Error::MaintainPitchUnsupported {
                element_id: clip.element_id.clone(),
                rate: clip.effective_rate(),
            });
        }
    }

    let range_seconds = (range.1 - range.0).to_seconds_f64();
    let out_frames = (range_seconds * f64::from(sample_rate)).ceil() as usize;
    let channel_count = channels as usize;
    let mut output = vec![0.0_f32; out_frames * channel_count];
    let range_start_seconds = range.0.to_seconds_f64();
    for clip in clips {
        mix_clip(
            clip,
            range_start_seconds,
            out_frames,
            sample_rate,
            channel_count,
            &mut output,
        );
    }
    Ok(output)
}

pub fn apply_mastering(samples: &mut [f32], channels: usize, sample_rate: u32) {
    let peak = samples.iter().fold(0.0_f32, |peak, s| peak.max(s.abs()));
    if f64::from(peak) <= MASTER_OUTPUT_HEADROOM {
        return;
    }
    let attack_coeff =
        (-1.0 / (MASTER_LIMITER_ATTACK_SECONDS * f64::from(sample_rate))).exp();
    let release_coeff =
        (-1.0 / (MASTER_LIMITER_RELEASE_SECONDS * f64::from(sample_rate))).exp();
    let mut envelope_db = 0.0_f64;
    for frame in samples.chunks_mut(channels) {
        let level = frame.iter().fold(0.0_f32, |level, s| level.max(s.abs()));
        let level_db = 20.0 * f64::from(level.max(1e-6)).log10();
        let over = level_db - MASTER_LIMITER_THRESHOLD_DB;
        let target_db = if over > 0.0 {
            -over * (1.0 - 1.0 / MASTER_LIMITER_RATIO)
        } else {
            0.0
        };
        let coeff = if target_db < envelope_db {
            attack_coeff
        } else {
            release_coeff
        };
        envelope_db = target_db + (envelope_db - target_db) * coeff;
        let gain = 10_f64.powf(envelope_db / 20.0) * MASTER_OUTPUT_HEADROOM;
        for sample in frame.iter_mut() {
            *sample = (f64::from(*sample) * gain) as f32;
        }
    }
    for sample in samples.iter_mut() {
        *sample = sample.clamp(-MASTER_OUTPUT_HEADROOM_F32, MASTER_OUTPUT_HEADROOM_F32);
    }
}

pub fn render_export_buffer(
    clips: &[AudibleClip],
    total_duration: MediaTime,
    sample_rate: u32,
    channels: u32,
) -> Result<Vec<f32>, Error> {
    if clips.is_empty() || total_duration <= MediaTime::ZERO {
        return Ok(Vec::new());
    }
    let mut buffer = mix_range(clips, (MediaTime::ZERO, total_duration), sample_rate, channels)?;
    apply_mastering(&mut buffer, channels as usize, sample_rate);
    Ok(buffer)
}
