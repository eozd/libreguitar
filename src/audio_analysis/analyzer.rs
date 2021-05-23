use crate::audio_analysis::algorithm::{find_note, moving_avg};
use crate::audio_analysis::analysis_result::AnalysisResult;
use crate::audio_analysis::target_notes::TargetNotes;
use crate::note::Note;
use crate::AudioCfg;
use realfft::{num_complex::Complex, RealFftPlanner, RealToComplex};
use std::f64;
use std::sync::Arc;

pub struct AudioAnalyzer {
    fft: Arc<dyn RealToComplex<f64>>,
    fft_buffer: Vec<f64>,
    fft_scratch: Vec<Complex<f64>>,
    spectrogram: Vec<Complex<f64>>,
    freq_magnitudes: Vec<f64>,
    fftsize: usize,
    n_bins: usize,
    delta_f: f64,
    target_notes: TargetNotes,
    audio_cfg: AudioCfg,
}

impl AudioAnalyzer {
    pub fn new(sample_rate: usize, target_notes: &[Note], audio_cfg: AudioCfg) -> AudioAnalyzer {
        assert!(
            target_notes.len() > 1,
            "Need at least two notes for analysis."
        );

        let target_notes = TargetNotes::new(Vec::from(target_notes));
        let min_freq_diff = target_notes.resolution();
        let delta_f = min_freq_diff / audio_cfg.fft_res_factor;
        let fftsize = (sample_rate as f64 / delta_f).ceil() as usize;

        let mut planner = RealFftPlanner::<f64>::new();
        let fft = planner.plan_fft_forward(fftsize);
        let fft_buffer = fft.make_input_vec();
        let spectrogram = fft.make_output_vec();
        let fft_scratch = fft.make_scratch_vec();
        let n_bins = spectrogram.len();
        let freq_magnitudes = vec![0.0f64; n_bins];
        AudioAnalyzer {
            fft,
            fft_buffer,
            fft_scratch,
            spectrogram,
            freq_magnitudes,
            fftsize,
            n_bins,
            delta_f,
            target_notes,
            audio_cfg,
        }
    }

    pub fn n_bins(&self) -> usize {
        self.n_bins
    }

    pub fn delta_f(&self) -> f64 {
        self.delta_f
    }

    fn compute_fft(&mut self, audio_data: impl ExactSizeIterator<Item = f64>) {
        let n_elems = audio_data.len();
        assert!(n_elems <= self.fft_buffer.len(), "Audio data is too long");
        for (i, val) in audio_data.enumerate() {
            self.fft_buffer[i] = val;
        }
        for i in n_elems..self.fft_buffer.len() {
            self.fft_buffer[i] = 0.0f64;
        }
        self.fft
            .process_with_scratch(
                &mut self.fft_buffer,
                &mut self.spectrogram,
                &mut self.fft_scratch,
            )
            .unwrap();
        let norm_factor = self.audio_cfg.fft_magnitude_gain / (self.fftsize as f64);
        for i in 0..self.n_bins {
            self.freq_magnitudes[i] = self.spectrogram[i].norm() * norm_factor;
        }
    }

    pub fn spectrogram(&self) -> &Vec<f64> {
        &self.freq_magnitudes
    }

    pub fn identify_note(
        &mut self,
        audio_data: impl ExactSizeIterator<Item = f64>,
    ) -> AnalysisResult {
        self.compute_fft(audio_data);
        moving_avg(
            &mut self.freq_magnitudes[..],
            self.audio_cfg.moving_avg_window_size,
        );
        let note = find_note(
            &self.freq_magnitudes,
            self.delta_f,
            &self.target_notes,
            self.audio_cfg.peak_threshold,
            self.audio_cfg.min_peak_dist,
            self.audio_cfg.num_top_peaks,
        );
        AnalysisResult { note }
    }
}
