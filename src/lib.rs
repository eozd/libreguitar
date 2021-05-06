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
            println!("Error reading data from device");
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
    buffer: Vec<Complex<f32>>,
    scratch: Vec<Complex<f32>>,
}

impl AudioAnalyzer {
    fn new() -> AudioAnalyzer {
        let buffer_size = 1024;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(buffer_size);
        let scratch_size = fft.get_inplace_scratch_len();
        let scratch = vec![
            Complex {
                re: 0.0f32,
                im: 0.0f32
            };
            scratch_size
        ];
        let buffer = vec![
            Complex {
                re: 0.0f32,
                im: 0.0f32
            };
            buffer_size
        ];

        AudioAnalyzer {
            fft,
            buffer,
            scratch,
        }
    }

    fn identify_note<'a>(
        &mut self,
        audio_data: &[f32],
        target_notes: &'a Vec<Note>,
    ) -> Option<&'a Note> {
        for i in 0..audio_data.len() {
            self.buffer[i].re = audio_data[i];
            self.buffer[i].im = 0.0f32;
        }
        self.fft
            .process_with_scratch(&mut self.buffer, &mut self.scratch);
        Some(&target_notes[0])
    }
}

struct GameLogic {
    title: String,
    fret_range: FretRange,
    string_range: StringRange,
    target_notes: Vec<Note>,
    analyzer: AudioAnalyzer,
}

impl GameLogic {
    fn new(
        title: String,
        fret_range: FretRange,
        string_range: StringRange,
        target_notes: Vec<Note>,
    ) -> GameLogic {
        GameLogic {
            title,
            fret_range,
            string_range,
            target_notes,
            analyzer: AudioAnalyzer::new(),
        }
    }

    fn tick(&mut self, audio_data: &[f32]) {
        if let Some(note) = self.analyzer.identify_note(audio_data, &self.target_notes) {
            println!("Detected note: {:?}", note);
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
