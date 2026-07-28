mod buckets;
mod clips;
mod mix;

pub use buckets::{
    extract_peak_range, extract_rms_buckets, extract_rms_range, extract_rms_windowed,
    sample_bucket_range,
};
pub use clips::{
    AudibleClip, clamp_db, clamp_retime_rate, collect_audible_elements, db_to_linear,
    source_time_at_clip_time,
};
pub use mix::{apply_mastering, mix_range, render_export_buffer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "element {element_id} requires pitch-preserving retime at rate {rate}, which is not supported"
    )]
    MaintainPitchUnsupported { element_id: String, rate: f64 },
    #[error("invalid time range: end must be after start")]
    InvalidTimeRange,
    #[error("sample rate must be at least 1")]
    InvalidSampleRate,
    #[error("channel count must be at least 1")]
    InvalidChannelCount,
}

pub type Result<T> = std::result::Result<T, Error>;
