use crate::note::Note;
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use statrs::statistics::Median;
use std::f64;
use std::sync::Arc;

pub struct AudioAnalyzer {
    fft: Arc<dyn Fft<f64>>,
    fft_buffer: Vec<Complex<f64>>,
    fft_scratch: Vec<Complex<f64>>,
    spectrogram: Vec<f64>,
    fftsize: usize,
    n_bins: usize,
    delta_f: f64,
    sample_rate: usize,
    target_notes: Vec<Note>,
}

impl AudioAnalyzer {
    pub fn new(sample_rate: usize, target_notes: Vec<Note>) -> AudioAnalyzer {
        assert!(
            target_notes.len() > 1,
            "Need at least two notes for analysis."
        );

        let min_freq_diff = target_notes[1].frequency - target_notes[0].frequency;
        let delta_f = min_freq_diff / 2.0;
        let fftsize = (sample_rate as f64 / delta_f).ceil() as usize;
        let n_bins = if fftsize % 2 == 0 {
            fftsize / 2 + 1
        } else {
            (fftsize + 1) / 2
        };

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fftsize);
        let fft_scratch = vec![
            Complex {
                re: 0.0f64,
                im: 0.0f64
            };
            fft.get_inplace_scratch_len()
        ];
        let fft_buffer = vec![
            Complex {
                re: 0.0f64,
                im: 0.0f64
            };
            fftsize
        ];

        let spectrogram = vec![0.0f64; n_bins];
        AudioAnalyzer {
            fft,
            fft_buffer,
            fft_scratch,
            spectrogram,
            fftsize,
            n_bins,
            delta_f,
            sample_rate,
            target_notes,
        }
    }

    fn compute_fft(&mut self, audio_data: &[f64]) {
        assert!(
            audio_data.len() <= self.fft_buffer.len(),
            "Audio data is too long"
        );
        for i in 0..audio_data.len() {
            self.fft_buffer[i].re = audio_data[i];
            self.fft_buffer[i].im = 0.0f64;
        }
        for i in audio_data.len()..self.fft_buffer.len() {
            self.fft_buffer[i].re = 0.0f64;
            self.fft_buffer[i].im = 0.0f64;
        }
        self.fft
            .process_with_scratch(&mut self.fft_buffer, &mut self.fft_scratch);
        let norm_factor = 1.0 / (self.fftsize as f64).sqrt();
        for i in 0..self.n_bins {
            self.spectrogram[i] = self.fft_buffer[i].norm() * norm_factor;
        }
    }

    pub fn identify_note<'a>(&'a mut self, audio_data: &[f64]) -> Option<&'a Note> {
        self.compute_fft(audio_data);
        moving_avg(&mut self.spectrogram[..], 10);
        print_freq(&self.spectrogram, self.delta_f, &self.target_notes);
        Some(&self.target_notes[0])
    }
}

fn print_freq(freq_spectrum: &[f64], delta_f: f64, target_notes: &Vec<Note>) {
    let median = freq_spectrum.median();
    let mut peaks = find_peaks(freq_spectrum, Some(25.0 * median), Some(25));
    peaks.sort_unstable_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
    let mut top_frequencies = [f64::NAN; 5];
    for (i, p) in peaks.iter().enumerate() {
        if i < 5 {
            top_frequencies[i] = (p.idx as f64) * delta_f;
        }
    }
    println!("Top 5 frequencies {:?}", &top_frequencies);
}

#[derive(Debug, PartialEq)]
struct Peak<T> {
    idx: usize,
    value: T,
}

impl<T> Peak<T> {
    fn new(idx: usize, value: T) -> Peak<T> {
        Peak { idx, value }
    }
}

fn find_peaks(
    signal: &[f64],
    min_height: Option<f64>,
    min_peak_dist: Option<usize>,
) -> Vec<Peak<f64>> {
    let n_samples = signal.len();
    if n_samples == 0 {
        return Vec::new();
    } else if n_samples == 1 {
        return vec![Peak::new(0, signal[0])];
    }
    let min_height = min_height.unwrap_or(0.0);
    let min_peak_dist = min_peak_dist.unwrap_or(0);
    let mut out: Vec<Peak<f64>> = Vec::new();
    for i in 0..n_samples {
        let greater_than_left = i == 0 || (signal[i] > signal[i - 1] && signal[i] >= min_height);
        let greater_than_right =
            i == n_samples - 1 || (signal[i] > signal[i + 1] && signal[i] >= min_height);
        if greater_than_left && greater_than_right && signal[i] >= min_height {
            if out.is_empty() || i - out[out.len() - 1].idx >= min_peak_dist {
                out.push(Peak::new(i, signal[i]));
            }
        }
    }
    out
}

fn moving_avg(signal: &mut [f64], window_size: usize) {
    assert!(
        window_size > 0,
        "Moving avg for zero window size is undefined."
    );
    if signal.len() == 0 || window_size == 1 {
        return;
    }
    let mut cumsum = vec![0.0f64; signal.len()];
    cumsum[0] = signal[0];
    for i in 1..signal.len() {
        cumsum[i] = cumsum[i - 1] + signal[i];
    }
    let left_offset = window_size / 2;
    let right_offset = window_size - 1 - left_offset;
    for i in (0..cumsum.len()).rev() {
        let right = (cumsum.len() - 1).min(i + right_offset);
        let avg = if i > left_offset {
            let left = i - left_offset - 1;
            (cumsum[right] - cumsum[left]) / (window_size as f64)
        } else {
            cumsum[right] / ((right + 1) as f64)
        };
        signal[i] = avg;
    }
}

#[cfg(test)]
mod tests_moving_avg {
    use super::moving_avg;

    #[test]
    fn moving_avg_empty_arr() {
        let mut signal = Vec::new();
        moving_avg(&mut signal, 1);
        assert_eq!(signal.len(), 0);

        moving_avg(&mut signal, 2);
        assert_eq!(signal.len(), 0);
    }

    #[test]
    #[should_panic]
    fn moving_avg_bad_window_size() {
        let mut signal = Vec::new();
        moving_avg(&mut signal, 0);
    }

    #[test]
    fn moving_avg_identity_case() {
        let mut signal = vec![1.0, 2.0, 3.0, 4.0];
        let expected = vec![1.0, 2.0, 3.0, 4.0];
        moving_avg(&mut signal, 1);
        assert_eq!(expected, signal);
    }

    #[test]
    fn moving_avg_general_case_0() {
        let mut signal = vec![1.0, 2.0, 3.0, 4.0];
        let expected = vec![1.0, 1.5, 2.5, 3.5];
        moving_avg(&mut signal, 2);
        assert_eq!(expected, signal);
    }

    #[test]
    fn moving_avg_general_case_1() {
        let mut signal = vec![1.0, 5.0, 10.0, 15.0, 7.5, -5.0];
        let expected = vec![
            3.0,
            16.0 / 3.0,
            30.0 / 3.0,
            32.5 / 3.0,
            17.5 / 3.0,
            2.5 / 3.0,
        ];
        moving_avg(&mut signal, 3);
        assert_eq!(expected, signal);
    }

    #[test]
    fn moving_avg_window_larger_than_arr_0() {
        let mut signal = vec![1.0];
        let expected = vec![1.0];
        moving_avg(&mut signal, 52);
        assert_eq!(expected, signal);
    }

    #[test]
    fn moving_avg_window_larger_than_arr_1() {
        let mut signal = vec![1.0, 5.0];
        let expected = vec![3.0, 3.0];
        moving_avg(&mut signal, 4);
        assert_eq!(expected, signal);
    }
}

#[cfg(test)]
mod tests_find_peaks {
    use super::{find_peaks, Peak};

    #[test]
    fn find_peaks_empty_arr() {
        let signal = Vec::new();
        let out = find_peaks(&signal, None, None);
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn find_peaks_single_elem() {
        let signal = vec![1.0];
        let expected = vec![Peak::new(0, 1.0)];
        let actual = find_peaks(&signal, None, None);
        assert_eq!(expected, actual);
    }

    #[test]
    fn find_peaks_decreasing() {
        let signal = vec![1.0, 0.5, 0.25];
        let expected = vec![Peak::new(0, 1.0)];
        let actual = find_peaks(&signal, None, None);
        assert_eq!(expected, actual);
    }

    #[test]
    fn find_peaks_parabola() {
        let signal = vec![1.0, 0.5, 0.25, 0.5, 1.0];
        let expected = vec![Peak::new(0, 1.0), Peak::new(4, 1.0)];
        let actual = find_peaks(&signal, None, None);
        assert_eq!(expected, actual);
    }

    #[test]
    fn find_peaks_two_peaks() {
        let signal = vec![0.5, 1.0, 2.0, 1.0, 0.0, 5.0, 2.5];
        let expected = vec![Peak::new(2, 2.0), Peak::new(5, 5.0)];
        let actual = find_peaks(&signal, None, None);
        assert_eq!(expected, actual);
    }
}
