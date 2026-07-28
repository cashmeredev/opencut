use std::collections::HashMap;
use std::path::Path;

use ffmpeg_next as ffmpeg;
use ffmpeg::codec::context::Context as CodecContext;
use ffmpeg::format::context::Input;
use ffmpeg::format::{Pixel, Sample, sample};
use ffmpeg::software::{resampling, scaling};
use ffmpeg::{ChannelLayout, Rational, media};
pub use time::MediaTime;

const MICROS_PER_SECOND: i64 = 1_000_000;
const AUDIO_PREROLL_SECONDS: f64 = 0.5;
const FORWARD_DECODE_LIMIT_SECONDS: f64 = 10.0;
const EOF_RESEEK_MARGIN_SECONDS: f64 = 2.0;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ffmpeg error: {0}")]
    Ffmpeg(#[from] ffmpeg::Error),
    #[error("no video stream found")]
    NoVideoStream,
    #[error("no audio stream found")]
    NoAudioStream,
    #[error("stream index {0} does not exist")]
    StreamNotFound(usize),
    #[error("stream index {0} is not a video stream")]
    NotAVideoStream(usize),
    #[error("stream index {0} is not an audio stream")]
    NotAnAudioStream(usize),
    #[error("no frames could be decoded from stream {0}")]
    NoFramesDecoded(usize),
    #[error("invalid time range: start must be before end")]
    InvalidTimeRange,
    #[error("bucket count must be at least 1")]
    InvalidBucketCount,
    #[error("max dimension must be at least 1")]
    InvalidMaxDimension,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct VideoStreamInfo {
    pub index: usize,
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
    pub frame_count: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct AudioStreamInfo {
    pub index: usize,
    pub channels: u32,
    pub sample_rate: u32,
}

#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub duration: MediaTime,
    pub video_streams: Vec<VideoStreamInfo>,
    pub audio_streams: Vec<AudioStreamInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct AudioSamples {
    pub sample_rate: u32,
    pub channels: u32,
    pub samples: Vec<f32>,
}

fn seconds_to_ts(seconds: f64, time_base: Rational) -> i64 {
    (seconds * f64::from(time_base.denominator()) / f64::from(time_base.numerator())).round()
        as i64
}

fn ts_to_seconds(ts: i64, time_base: Rational) -> f64 {
    ts as f64 * f64::from(time_base.numerator()) / f64::from(time_base.denominator())
}

fn seconds_to_micros(seconds: f64) -> i64 {
    (seconds * MICROS_PER_SECOND as f64) as i64
}

fn effective_layout(layout: ChannelLayout, channels: u16) -> ChannelLayout {
    if layout.is_empty() {
        ChannelLayout::default(i32::from(channels))
    } else {
        layout
    }
}

struct VideoPipeline {
    stream_index: usize,
    decoder: ffmpeg::codec::decoder::Video,
    time_base: Rational,
    last: Option<(i64, ffmpeg::frame::Video)>,
    drained: bool,
    scaler: Option<(Pixel, u32, u32, scaling::Context)>,
}

impl VideoPipeline {
    fn open(input: &Input, stream_index: usize) -> Result<Self> {
        let stream = input
            .streams()
            .find(|stream| stream.index() == stream_index)
            .ok_or(Error::StreamNotFound(stream_index))?;
        if stream.parameters().medium() != media::Type::Video {
            return Err(Error::NotAVideoStream(stream_index));
        }
        let decoder = CodecContext::from_parameters(stream.parameters())?
            .decoder()
            .video()?;
        Ok(Self {
            stream_index,
            decoder,
            time_base: stream.time_base(),
            last: None,
            drained: false,
            scaler: None,
        })
    }

    fn frame_at(
        &mut self,
        input: &mut Input,
        target_ts: i64,
        seek_seconds: f64,
    ) -> Result<ffmpeg::frame::Video> {
        if let Some((last_pts, last_frame)) = &self.last {
            if target_ts >= *last_pts {
                if self.drained {
                    return Ok(last_frame.clone());
                }
                let forward_limit =
                    seconds_to_ts(FORWARD_DECODE_LIMIT_SECONDS, self.time_base);
                if target_ts - last_pts <= forward_limit {
                    let candidate = self.last.take();
                    return self.decode_forward(input, target_ts, candidate);
                }
            }
        }
        self.decoder.flush();
        self.drained = false;
        self.last = None;
        let micros = seconds_to_micros(seek_seconds.max(0.0));
        input.seek(micros, ..micros)?;
        match self.decode_forward(input, target_ts, None) {
            Err(Error::NoFramesDecoded(_)) => {
                self.decoder.flush();
                input.seek(0, ..0)?;
                self.decode_forward(input, i64::MAX, None)
            }
            result => result,
        }
    }

    fn decode_forward(
        &mut self,
        input: &mut Input,
        target_ts: i64,
        mut candidate: Option<(i64, ffmpeg::frame::Video)>,
    ) -> Result<ffmpeg::frame::Video> {
        let mut frame = ffmpeg::frame::Video::empty();
        for (stream, packet) in input.packets() {
            if stream.index() != self.stream_index {
                continue;
            }
            self.decoder.send_packet(&packet)?;
            while self.decoder.receive_frame(&mut frame).is_ok() {
                let pts = frame.pts().unwrap_or(i64::MIN);
                if pts <= target_ts {
                    candidate = Some((pts, frame.clone()));
                } else {
                    self.last = Some((pts, frame.clone()));
                    return match candidate {
                        Some((_, frame)) => Ok(frame),
                        None => Ok(frame.clone()),
                    };
                }
                frame = ffmpeg::frame::Video::empty();
            }
        }
        self.decoder.send_eof()?;
        while self.decoder.receive_frame(&mut frame).is_ok() {
            let pts = frame.pts().unwrap_or(i64::MIN);
            if pts <= target_ts {
                candidate = Some((pts, frame.clone()));
            } else {
                self.last = Some((pts, frame.clone()));
                return match candidate {
                    Some((_, frame)) => Ok(frame),
                    None => Ok(frame.clone()),
                };
            }
            frame = ffmpeg::frame::Video::empty();
        }
        self.drained = true;
        match candidate {
            Some((pts, frame)) => {
                self.last = Some((pts, frame.clone()));
                Ok(frame)
            }
            None => Err(Error::NoFramesDecoded(self.stream_index)),
        }
    }

    fn rgba_frame(&mut self, frame: &ffmpeg::frame::Video) -> Result<VideoFrame> {
        convert_to_rgba(
            &mut self.scaler,
            frame,
            frame.width(),
            frame.height(),
        )
    }
}

fn convert_to_rgba(
    cache: &mut Option<(Pixel, u32, u32, scaling::Context)>,
    frame: &ffmpeg::frame::Video,
    width: u32,
    height: u32,
) -> Result<VideoFrame> {
    let key = (frame.format(), frame.width(), frame.height());
    let matches = cache
        .as_ref()
        .is_some_and(|(format, w, h, _)| (*format, *w, *h) == key);
    if !matches {
        let scaler = scaling::Context::get(
            frame.format(),
            frame.width(),
            frame.height(),
            Pixel::RGBA,
            width,
            height,
            scaling::Flags::BILINEAR,
        )?;
        *cache = Some((key.0, key.1, key.2, scaler));
    }
    let scaler = &mut cache.as_mut().expect("scaler just inserted").3;
    let mut converted = ffmpeg::frame::Video::new(Pixel::RGBA, width, height);
    scaler.run(frame, &mut converted)?;
    let stride = converted.stride(0);
    let row_bytes = width as usize * 4;
    let plane = converted.data(0);
    let mut data = Vec::with_capacity(row_bytes * height as usize);
    if stride == row_bytes {
        data.extend_from_slice(&plane[..row_bytes * height as usize]);
    } else {
        for row in 0..height as usize {
            data.extend_from_slice(&plane[row * stride..row * stride + row_bytes]);
        }
    }
    Ok(VideoFrame { width, height, data })
}

pub struct Decoder {
    input: Input,
    video: HashMap<usize, VideoPipeline>,
}

impl Decoder {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let input = ffmpeg::format::input(path.as_ref())?;
        Ok(Self {
            input,
            video: HashMap::new(),
        })
    }

    pub fn info(&self) -> MediaInfo {
        let mut video_streams = Vec::new();
        let mut audio_streams = Vec::new();
        let mut stream_duration_seconds = 0.0_f64;
        for stream in self.input.streams() {
            let duration = stream.duration();
            if duration > 0 {
                stream_duration_seconds =
                    stream_duration_seconds.max(ts_to_seconds(duration, stream.time_base()));
            }
            match stream.parameters().medium() {
                media::Type::Video => {
                    let Ok(context) = CodecContext::from_parameters(stream.parameters()) else {
                        continue;
                    };
                    let Ok(decoder) = context.decoder().video() else {
                        continue;
                    };
                    let rate = stream.avg_frame_rate();
                    let frame_rate = if rate.denominator() == 0 {
                        0.0
                    } else {
                        f64::from(rate.numerator()) / f64::from(rate.denominator())
                    };
                    let frames = stream.frames();
                    video_streams.push(VideoStreamInfo {
                        index: stream.index(),
                        width: decoder.width(),
                        height: decoder.height(),
                        frame_rate,
                        frame_count: (frames > 0).then_some(frames as u64),
                    });
                }
                media::Type::Audio => {
                    let Ok(context) = CodecContext::from_parameters(stream.parameters()) else {
                        continue;
                    };
                    let Ok(decoder) = context.decoder().audio() else {
                        continue;
                    };
                    audio_streams.push(AudioStreamInfo {
                        index: stream.index(),
                        channels: u32::from(decoder.channels()),
                        sample_rate: decoder.rate(),
                    });
                }
                _ => {}
            }
        }
        let duration_micros = self.input.duration();
        let duration_seconds = if duration_micros > 0 {
            duration_micros as f64 / MICROS_PER_SECOND as f64
        } else {
            stream_duration_seconds
        };
        MediaInfo {
            duration: MediaTime::from_seconds_f64(duration_seconds).unwrap_or(MediaTime::ZERO),
            video_streams,
            audio_streams,
        }
    }

    pub fn decode_video_frame(&mut self, stream_index: usize, time: MediaTime) -> Result<VideoFrame> {
        let raw = self.decode_video_raw(stream_index, time)?;
        let pipeline = self
            .video
            .get_mut(&stream_index)
            .expect("pipeline created by decode_video_raw");
        pipeline.rgba_frame(&raw)
    }

    fn decode_video_raw(
        &mut self,
        stream_index: usize,
        time: MediaTime,
    ) -> Result<ffmpeg::frame::Video> {
        if !self.video.contains_key(&stream_index) {
            let pipeline = VideoPipeline::open(&self.input, stream_index)?;
            self.video.insert(stream_index, pipeline);
        }
        let target_seconds = time.to_seconds_f64().max(0.0);
        let duration_seconds = self.input.duration().max(0) as f64 / MICROS_PER_SECOND as f64;
        let pipeline = self
            .video
            .get_mut(&stream_index)
            .expect("pipeline just inserted");
        let mut target_ts = seconds_to_ts(target_seconds, pipeline.time_base);
        let mut seek_seconds = target_seconds;
        if duration_seconds > 0.0 && target_seconds > duration_seconds {
            target_ts = i64::MAX;
            seek_seconds = (duration_seconds - EOF_RESEEK_MARGIN_SECONDS).max(0.0);
        }
        pipeline.frame_at(&mut self.input, target_ts, seek_seconds)
    }

    pub fn decode_audio_range(
        &mut self,
        stream_index: usize,
        start: MediaTime,
        end: MediaTime,
    ) -> Result<AudioSamples> {
        self.decode_audio_range_inner(stream_index, start, end, true)
    }

    fn decode_audio_range_inner(
        &mut self,
        stream_index: usize,
        start: MediaTime,
        end: MediaTime,
        pad_to_range: bool,
    ) -> Result<AudioSamples> {
        if end <= start {
            return Err(Error::InvalidTimeRange);
        }
        let stream = self
            .input
            .streams()
            .find(|stream| stream.index() == stream_index)
            .ok_or(Error::StreamNotFound(stream_index))?;
        if stream.parameters().medium() != media::Type::Audio {
            return Err(Error::NotAnAudioStream(stream_index));
        }
        let time_base = stream.time_base();
        let mut decoder = CodecContext::from_parameters(stream.parameters())?
            .decoder()
            .audio()?;
        let rate = decoder.rate();
        let channels = u32::from(decoder.channels());
        let layout = effective_layout(decoder.channel_layout(), decoder.channels());
        let mut resampler = resampling::Context::get(
            decoder.format(),
            layout,
            rate,
            Sample::F32(sample::Type::Packed),
            layout,
            rate,
        )?;
        let start_seconds = start.to_seconds_f64().max(0.0);
        let end_seconds = end.to_seconds_f64();
        let seek_seconds = (start_seconds - AUDIO_PREROLL_SECONDS).max(0.0);
        let seek_micros = seconds_to_micros(seek_seconds);
        self.input.seek(seek_micros, ..seek_micros)?;

        let rate_f64 = f64::from(rate);
        let channel_count = channels as usize;
        let mut anchor_seconds: Option<f64> = None;
        let mut produced: i64 = 0;
        let mut out_samples: Vec<f32> = Vec::new();
        let mut done = false;

        let mut frame = ffmpeg::frame::Audio::empty();
        for (packet_stream, packet) in self.input.packets() {
            if packet_stream.index() != stream_index {
                continue;
            }
            decoder.send_packet(&packet)?;
            while decoder.receive_frame(&mut frame).is_ok() {
                if frame.channel_layout().is_empty() {
                    frame.set_channel_layout(layout);
                }
                if anchor_seconds.is_none() {
                    anchor_seconds = Some(match frame.pts() {
                        Some(pts) => ts_to_seconds(pts, time_base),
                        None => seek_seconds,
                    });
                }
                let mut converted = ffmpeg::frame::Audio::empty();
                resampler.run(&frame, &mut converted)?;
                append_resampled(
                    &converted,
                    anchor_seconds.expect("anchor set above"),
                    start_seconds,
                    end_seconds,
                    rate_f64,
                    channel_count,
                    &mut produced,
                    &mut out_samples,
                );
                frame = ffmpeg::frame::Audio::empty();
                let anchor = anchor_seconds.expect("anchor set above");
                let end_index = ((end_seconds - anchor) * rate_f64).round() as i64;
                if produced >= end_index {
                    done = true;
                    break;
                }
            }
            if done {
                break;
            }
        }
        if !done {
            decoder.send_eof()?;
            while decoder.receive_frame(&mut frame).is_ok() {
                if frame.channel_layout().is_empty() {
                    frame.set_channel_layout(layout);
                }
                if anchor_seconds.is_none() {
                    anchor_seconds = Some(match frame.pts() {
                        Some(pts) => ts_to_seconds(pts, time_base),
                        None => seek_seconds,
                    });
                }
                let mut converted = ffmpeg::frame::Audio::empty();
                resampler.run(&frame, &mut converted)?;
                append_resampled(
                    &converted,
                    anchor_seconds.expect("anchor set above"),
                    start_seconds,
                    end_seconds,
                    rate_f64,
                    channel_count,
                    &mut produced,
                    &mut out_samples,
                );
                frame = ffmpeg::frame::Audio::empty();
            }
            let mut converted = ffmpeg::frame::Audio::empty();
            if resampler.flush(&mut converted).is_ok() && converted.samples() > 0 {
                append_resampled(
                    &converted,
                    anchor_seconds.unwrap_or(seek_seconds),
                    start_seconds,
                    end_seconds,
                    rate_f64,
                    channel_count,
                    &mut produced,
                    &mut out_samples,
                );
            }
        }
        if pad_to_range && let Some(anchor) = anchor_seconds {
            let start_index = (((start_seconds - anchor) * rate_f64).round() as i64).max(0);
            let end_index = (((end_seconds - anchor) * rate_f64).round() as i64).max(start_index);
            let expected = ((end_index - start_index) as usize) * channel_count;
            if out_samples.len() < expected {
                out_samples.resize(expected, 0.0);
            }
        }
        Ok(AudioSamples {
            sample_rate: rate,
            channels,
            samples: out_samples,
        })
    }
}

fn append_resampled(
    converted: &ffmpeg::frame::Audio,
    anchor_seconds: f64,
    start_seconds: f64,
    end_seconds: f64,
    rate: f64,
    channels: usize,
    produced: &mut i64,
    out_samples: &mut Vec<f32>,
) {
    let count = converted.samples() as i64;
    if count == 0 {
        return;
    }
    let start_index = (((start_seconds - anchor_seconds) * rate).round() as i64).max(0);
    let end_index = (((end_seconds - anchor_seconds) * rate).round() as i64).max(start_index);
    let low = (*produced).max(start_index);
    let high = (*produced + count).min(end_index);
    if low < high {
        if out_samples.is_empty() && low > start_index {
            out_samples.resize(((low - start_index) as usize) * channels, 0.0);
        }
        let data = converted.data(0);
        let first = (low - *produced) as usize * channels;
        let last = (high - *produced) as usize * channels;
        for bytes in data[first * 4..last * 4].chunks_exact(4) {
            out_samples.push(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
        }
    }
    *produced += count;
}

pub fn thumbnail(path: impl AsRef<Path>, max_dimension: u32) -> Result<VideoFrame> {
    if max_dimension == 0 {
        return Err(Error::InvalidMaxDimension);
    }
    let mut decoder = Decoder::open(path.as_ref())?;
    let video_stream = decoder
        .info()
        .video_streams
        .first()
        .cloned()
        .ok_or(Error::NoVideoStream)?;
    let raw = decoder.decode_video_raw(video_stream.index, MediaTime::ZERO)?;
    let source_max = raw.width().max(raw.height());
    let scale = (f64::from(max_dimension) / f64::from(source_max)).min(1.0);
    let width = ((f64::from(raw.width()) * scale).round() as u32).max(1);
    let height = ((f64::from(raw.height()) * scale).round() as u32).max(1);
    let mut cache = None;
    convert_to_rgba(&mut cache, &raw, width, height)
}

pub fn waveform_summary(path: impl AsRef<Path>, bucket_count: usize) -> Result<Vec<(f32, f32)>> {
    if bucket_count == 0 {
        return Err(Error::InvalidBucketCount);
    }
    let mut decoder = Decoder::open(path.as_ref())?;
    let info = decoder.info();
    let audio_stream = info.audio_streams.first().ok_or(Error::NoAudioStream)?;
    let samples = decoder.decode_audio_range_inner(
        audio_stream.index,
        MediaTime::ZERO,
        info.duration,
        false,
    )?;
    let channels = samples.channels as usize;
    let frame_count = samples.samples.len() / channels;
    let bucket_size = frame_count.div_ceil(bucket_count).max(1);
    Ok((0..bucket_count)
        .map(|bucket| {
            let start_frame = bucket * bucket_size;
            let end_frame = (start_frame + bucket_size).min(frame_count);
            if start_frame >= end_frame {
                return (0.0, 0.0);
            }
            let slice = &samples.samples[start_frame * channels..end_frame * channels];
            let mut low = f32::INFINITY;
            let mut high = f32::NEG_INFINITY;
            for &sample in slice {
                low = low.min(sample);
                high = high.max(sample);
            }
            (low, high)
        })
        .collect())
}
