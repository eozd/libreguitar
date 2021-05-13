use crate::note::Note;
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use statrs::statistics::Median;
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
    let mut maxval = 0.0f64;
    let mut maxidx = 0;
    for i in 0..freq_spectrum.len() {
        let currval = freq_spectrum[i];
        if currval > maxval {
            maxval = currval;
            maxidx = i;
        }
    }
    let max_freq = delta_f * (maxidx as f64);
    println!("Median frequency {} Highest frequency {}", median, max_freq);
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
mod tests {
    use super::*;

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
