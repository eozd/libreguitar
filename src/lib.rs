use cpal::traits::DeviceTrait;
use cpal::traits::StreamTrait;
use cpal::BuildStreamError;
use cpal::Device;
use cpal::Stream;
use cpal::StreamConfig;
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use serde::Deserialize;
use std::error::Error;
use std::sync::Arc;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error(transparent)]
    BuildStreamError(#[from] cpal::BuildStreamError),
    #[error(transparent)]
    PlayStreamError(#[from] cpal::PlayStreamError),
    #[error(transparent)]
    UnknownError(#[from] Box<dyn Error>),
}

const GAME_TITLE: &str = "FRETBOARD TRAINER";
const MAX_FRETS: usize = 24;
const MAX_STRINGS: usize = 6;

pub fn run(device: Device, config: StreamConfig, freq_csv_path: &str) -> Result<(), GameError> {
    let note_vec = parse_freq_csv(freq_csv_path)?;
    let game = GameLogic::new(
        String::from(GAME_TITLE),
        FretRange::new(0, 12),
        StringRange::new(1, 6 + 1),
        config.sample_rate.0 as usize,
        note_vec,
    );

    let stream = build_stream(&device, &config, game)?;
    println!("Playing device...");
    stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(1000));
    Ok(())
}

fn build_stream(
    device: &Device,
    config: &StreamConfig,
    mut game: GameLogic,
) -> Result<Stream, BuildStreamError> {
    device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            game.tick(data);
        },
        move |_err| {
            // println!("Error reading data from device {}", _err);
        },
    )
}

fn parse_freq_csv(csv_path: &str) -> Result<Vec<Note>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(csv_path)?;
    let mut iter = rdr.deserialize();
    let mut out = Vec::new();
    while let Some(result) = iter.next() {
        out.push(result?);
    }
    Ok(out)
}

#[derive(Debug, Deserialize)]
enum NoteName {
    A,
    ASharp,
    B,
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
}

#[derive(Debug, Deserialize)]
struct Note {
    octave: usize,
    name: NoteName,
    frequency: f32,
}

struct FretRange {
    beg_fret: usize,
    end_fret: usize,
}

impl FretRange {
    fn new(beg_fret: usize, end_fret: usize) -> FretRange {
        assert!(
            beg_fret <= MAX_FRETS && end_fret <= MAX_FRETS + 1,
            "Maximum {} fret guitars are supported.",
            MAX_FRETS
        );
        assert!(
            beg_fret < end_fret,
            "Fret range must include at least one fret."
        );

        FretRange { beg_fret, end_fret }
    }
}

struct StringRange {
    beg_string: usize,
    end_string: usize,
}

impl StringRange {
    fn new(beg_string: usize, end_string: usize) -> StringRange {
        assert!(
            beg_string <= MAX_STRINGS && end_string <= MAX_STRINGS + 1,
            "Maximum {} string guitars are supported.",
            MAX_STRINGS
        );
        assert!(beg_string >= 1);
        assert!(
            beg_string < end_string,
            "String range must include at least one string."
        );

        StringRange {
            beg_string,
            end_string,
        }
    }
}

struct AudioAnalyzer {
    fft: Arc<dyn Fft<f32>>,
    fft_buffer: Vec<Complex<f32>>,
    fft_scratch: Vec<Complex<f32>>,
    delta_f: f32,
    sample_rate: usize,
    target_notes: Vec<Note>,
}

impl AudioAnalyzer {
    fn new(sample_rate: usize, target_notes: Vec<Note>) -> AudioAnalyzer {
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
        let mut maxval = freq_spectrum[0].re;
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

    fn identify_note<'a>(&'a mut self, audio_data: &[f32]) -> Option<&'a Note> {
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

struct GameLogic {
    title: String,
    fret_range: FretRange,
    string_range: StringRange,
    analyzer: AudioAnalyzer,
}

impl GameLogic {
    fn new(
        title: String,
        fret_range: FretRange,
        string_range: StringRange,
        sample_rate: usize,
        target_notes: Vec<Note>,
    ) -> GameLogic {
        GameLogic {
            title,
            fret_range,
            string_range,
            analyzer: AudioAnalyzer::new(sample_rate, target_notes),
        }
    }

    fn tick(&mut self, audio_data: &[f32]) {
        // println!(
        //     "Channel abs sum {}",
        //     audio_data.iter().map(|x| x.abs()).sum::<f32>()
        // );

        if let Some(note) = self.analyzer.identify_note(audio_data) {
            // println!("Read {} floats", audio_data.len());
            // println!("Detected note: {:?}", note);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
