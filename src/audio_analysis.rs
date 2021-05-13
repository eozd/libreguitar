use crate::note::Note;
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use std::sync::Arc;

pub struct AudioAnalyzer {
    fft: Arc<dyn Fft<f32>>,
    fft_buffer: Vec<Complex<f32>>,
    fft_scratch: Vec<Complex<f32>>,
    delta_f: f32,
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
        let fftsize = (sample_rate as f32 / delta_f).ceil() as usize;

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fftsize);
        let fft_scratch = vec![
            Complex {
                re: 0.0f32,
                im: 0.0f32
            };
            fft.get_inplace_scratch_len()
        ];
        let fft_buffer = vec![
            Complex {
                re: 0.0f32,
                im: 0.0f32
            };
            fftsize
        ];

        AudioAnalyzer {
            fft,
            fft_buffer,
            fft_scratch,
            delta_f,
            sample_rate,
            target_notes,
        }
    }

    fn compute_fft(&mut self, audio_data: &[f32]) {
        assert!(
            audio_data.len() <= self.fft_buffer.len(),
            "Audio data is too long"
        );
        for i in 0..audio_data.len() {
            self.fft_buffer[i].re = audio_data[i];
            self.fft_buffer[i].im = 0.0f32;
        }
        for i in audio_data.len()..self.fft_buffer.len() {
            self.fft_buffer[i].re = 0.0f32;
            self.fft_buffer[i].im = 0.0f32;
        }
        self.fft
            .process_with_scratch(&mut self.fft_buffer, &mut self.fft_scratch);
    }

    fn print_freq(&self, freq_spectrum: &[Complex<f32>]) {
        let mut maxval = 0.0f32;
        let mut maxidx = 0;
        for i in 0..freq_spectrum.len() {
            let currval = freq_spectrum[i].norm_sqr();
            if currval > maxval {
                maxval = currval;
                maxidx = i;
            }
        }
        let max_freq = self.delta_f * (maxidx as f32);
        println!("Highest frequency {}", max_freq);
    }

    pub fn identify_note<'a>(&'a mut self, audio_data: &[f32]) -> Option<&'a Note> {
        self.compute_fft(audio_data);
        let fftsize = self.fft_buffer.len();
        let n_bins = if fftsize % 2 == 0 {
            fftsize / 2 + 1
        } else {
            (fftsize + 1) / 2
        };

        self.print_freq(&self.fft_buffer[..n_bins]);
        Some(&self.target_notes[0])
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
