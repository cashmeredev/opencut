pub const RMS_ANALYSIS_WINDOW_SECONDS: f64 = 0.02;

pub fn sample_bucket_range(
    start_sample: usize,
    end_sample: usize,
    bucket_index: usize,
    bucket_count: usize,
) -> (usize, usize) {
    let range_length = end_sample.saturating_sub(start_sample);
    let bucket_start = start_sample + bucket_index * range_length / bucket_count;
    let bucket_end = start_sample + (bucket_index + 1) * range_length / bucket_count;
    (bucket_start, bucket_end.max(bucket_start))
}

pub fn extract_peak_range(
    samples: &[f32],
    channels: usize,
    count: usize,
    start_sample: usize,
    end_sample: usize,
) -> Vec<f32> {
    let channels = channels.max(1);
    let frames = samples.len() / channels;
    let end = end_sample.min(frames);
    let start = start_sample.min(end);
    let mut peaks = vec![0.0_f32; count];
    for (index, peak) in peaks.iter_mut().enumerate() {
        let (bucket_start, bucket_end) = sample_bucket_range(start, end, index, count);
        let mut bucket_peak = 0.0_f32;
        for frame in bucket_start..bucket_end {
            for channel in 0..channels {
                let magnitude = samples[frame * channels + channel].abs();
                if magnitude > bucket_peak {
                    bucket_peak = magnitude;
                }
            }
        }
        *peak = bucket_peak;
    }
    peaks
}

pub fn extract_rms_windowed(
    samples: &[f32],
    channels: usize,
    sample_rate: u32,
    buckets: &[(usize, usize)],
) -> Vec<f32> {
    let channels = channels.max(1);
    let max_window =
        (f64::from(sample_rate) * RMS_ANALYSIS_WINDOW_SECONDS).floor().max(1.0) as usize;
    buckets
        .iter()
        .map(|&(bucket_start, bucket_end)| {
            let bucket_length = bucket_end.saturating_sub(bucket_start);
            if bucket_length == 0 {
                return 0.0;
            }
            let window_length = bucket_length.min(max_window).max(1);
            let mut max_mean_square = 0.0_f64;
            let mut window_start = bucket_start;
            while window_start < bucket_end {
                let window_end = (window_start + window_length).min(bucket_end);
                let n = window_end - window_start;
                let mut sum = 0.0_f64;
                for frame in window_start..window_end {
                    for channel in 0..channels {
                        let value = f64::from(samples[frame * channels + channel]);
                        sum += value * value;
                    }
                }
                let mean_square = sum / (n * channels) as f64;
                if mean_square > max_mean_square {
                    max_mean_square = mean_square;
                }
                window_start = window_end;
            }
            max_mean_square.sqrt() as f32
        })
        .collect()
}

pub fn extract_rms_range(
    samples: &[f32],
    channels: usize,
    sample_rate: u32,
    count: usize,
    start_sample: usize,
    end_sample: usize,
) -> Vec<f32> {
    let buckets: Vec<(usize, usize)> = (0..count)
        .map(|index| sample_bucket_range(start_sample, end_sample, index, count))
        .collect();
    extract_rms_windowed(samples, channels, sample_rate, &buckets)
}

pub fn extract_rms_buckets(
    samples: &[f32],
    channels: usize,
    sample_rate: u32,
    bucket_count: usize,
) -> Vec<f32> {
    let frames = samples.len() / channels.max(1);
    extract_rms_range(samples, channels, sample_rate, bucket_count, 0, frames)
}
