use crate::audio_analysis::AudioAnalyzer;
use crate::note::Note;
use std::error::Error;
use thiserror::Error;

const MAX_FRETS: usize = 24;
const MAX_STRINGS: usize = 6;

#[derive(Error, Debug)]
pub enum GameError {
    #[error(transparent)]
    BuildStreamError(#[from] cpal::BuildStreamError),
    #[error(transparent)]
    PlayStreamError(#[from] cpal::PlayStreamError),
    #[error(transparent)]
    UnknownError(#[from] Box<dyn Error>),
}

pub struct FretRange {
    beg_fret: usize,
    end_fret: usize,
}

impl FretRange {
    pub fn new(beg_fret: usize, end_fret: usize) -> FretRange {
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

pub struct StringRange {
    beg_string: usize,
    end_string: usize,
}

impl StringRange {
    pub fn new(beg_string: usize, end_string: usize) -> StringRange {
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

pub struct GameLogic {
    title: String,
    fret_range: FretRange,
    string_range: StringRange,
    analyzer: AudioAnalyzer,
}

impl GameLogic {
    pub fn new(
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

    pub fn tick(&mut self, audio_data: &[f32]) {
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
