use cpal::traits::DeviceTrait;
use cpal::traits::StreamTrait;
use cpal::BuildStreamError;
use cpal::Device;
use cpal::Stream;
use cpal::StreamConfig;
use serde::Deserialize;
use std::error::Error;
use std::str::FromStr;

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
    let freq_vec: Vec<f32> = note_vec.iter().map(|note| note.frequency).collect();

    let game = GameLogic::new(
        String::from(GAME_TITLE),
        FretRange::new(0, 12),
        StringRange::new(1, 6 + 1),
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

impl Note {
    fn new(name: NoteName, octave: usize, frequency: f32) -> Note {
        assert!(frequency >= 0.0, "Frequency must be nonnegative");
        Note {
            name,
            octave,
            frequency,
        }
    }
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

struct GameLogic {
    title: String,
    fret_range: FretRange,
    string_range: StringRange,
    frame_cnt: usize,
}

impl GameLogic {
    fn new(title: String, fret_range: FretRange, string_range: StringRange) -> GameLogic {
        GameLogic {
            title,
            fret_range,
            string_range,
            frame_cnt: 0,
        }
    }

    fn tick(&mut self, audio_data: &[f32]) {
        println!(
            "Tick {}: Maximum is {:?}",
            self.frame_cnt,
            audio_data.iter().cloned().fold(0. / 0., f32::max)
        );
        self.frame_cnt += 1;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
