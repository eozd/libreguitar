use crate::core::csv::parse_csv;
use crate::core::{Note, NoteName, NoteRegistry};
use serde::Deserialize;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct InvalidTuningError(String);
impl fmt::Display for InvalidTuningError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InvalidTuningError: {}", self.0)
    }
}
impl Error for InvalidTuningError {}

#[derive(Deserialize, PartialEq)]
struct TuningSpecification {
    string: usize,
    octave: usize,
    name: NoteName,
}

#[derive(Clone)]
pub struct Tuning {
    values: Vec<Note>,
}

impl Tuning {
    pub fn from_csv(
        csv_path: &str,
        note_registry: &NoteRegistry,
    ) -> Result<Tuning, Box<dyn Error>> {
        let tuning_spec: Vec<TuningSpecification> = parse_csv(csv_path)?;

        match Tuning::from_specification(&tuning_spec[..], note_registry) {
            Ok(v) => Ok(v),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn from_specification(
        tuning_spec: &[TuningSpecification],
        note_registry: &NoteRegistry,
    ) -> Result<Tuning, InvalidTuningError> {
        let mut map = Vec::new();
        for (i, row) in tuning_spec.iter().enumerate() {
            if row.string - 1 != i {
                return Err(InvalidTuningError(String::from(
                    "Tuning specification needs strings to be numbered as 1, 2, 3, ...",
                )));
            }
            if let Some(note) = note_registry.get(row.name, row.octave) {
                map.push(note.clone());
            } else {
                return Err(InvalidTuningError(String::from(
                    "Tuning specification contains a note not given in note frequency list",
                )));
            }
        }
        if map.is_empty() {
            return Err(InvalidTuningError(String::from(
                "Tuning specification needs at least one string",
            )));
        }
        Ok(Tuning { values: map })
    }

    pub fn note(&self, string_idx: usize) -> &Note {
        debug_assert!(
            string_idx > 0 && string_idx <= self.values.len(),
            "Guitar string index {} is out of bounds ({}, {})",
            string_idx,
            1,
            self.values.len() + 1
        );
        &self.values[string_idx - 1]
    }

    pub fn iter(&self) -> impl Iterator<Item = &Note> {
        self.values.iter()
    }
}
