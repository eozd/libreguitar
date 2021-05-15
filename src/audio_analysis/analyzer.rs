use crate::audio_analysis::algorithm::{find_note, moving_avg};
use crate::audio_analysis::analysis_result::AnalysisResult;
use crate::audio_analysis::target_notes::TargetNotes;
use crate::note::Note;
use rustfft::{num_complex::Complex, Fft, FftPlanner};
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
    target_notes: TargetNotes,
}

impl AudioAnalyzer {
    pub fn new(sample_rate: usize, target_notes: Vec<Note>) -> AudioAnalyzer {
        assert!(
            target_notes.len() > 1,
            "Need at least two notes for analysis."
        );

        let target_notes = TargetNotes::new(target_notes);
        let min_freq_diff = target_notes.resolution();
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
            target_notes,
        }
    }

    pub fn n_bins(&self) -> usize {
        self.n_bins
    }

    pub fn delta_f(&self) -> f64 {
        self.delta_f
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
        let norm_factor = 10.0 / (self.fftsize as f64);
        for i in 0..self.n_bins {
            self.spectrogram[i] = self.fft_buffer[i].norm() * norm_factor;
        }
    }

    pub fn identify_note(&mut self, audio_data: &[f64]) -> AnalysisResult {
        self.compute_fft(audio_data);
        moving_avg(&mut self.spectrogram[..], 10);
        let note = find_note(&self.spectrogram, self.delta_f, &self.target_notes);
        AnalysisResult {
            note: note,
            spectrogram: &self.spectrogram,
        }
    }
}
