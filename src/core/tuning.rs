use crate::core::csv::parse_csv;
use crate::core::{Note, NoteName, NoteRegistry};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct InvalidTuningError(String);
impl fmt::Display for InvalidTuningError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InvalidTuningError: {}", self.0)
    }
}
impl Error for InvalidTuningError {}

#[derive(Deserialize, PartialEq)]
pub struct TuningSpecification {
    pub string: usize,
    pub octave: i32,
    pub name: NoteName,
}

#[derive(Clone)]
pub struct Tuning {
    values: BTreeMap<usize, Note>,
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

    pub fn from_specification(
        tuning_spec: &[TuningSpecification],
        note_registry: &NoteRegistry,
    ) -> Result<Tuning, InvalidTuningError> {
        let mut map = BTreeMap::new();
        for row in tuning_spec.iter() {
            if let Some(note) = note_registry.get(row.name, row.octave) {
                map.insert(row.string, note.clone());
            } else {
                return Err(InvalidTuningError(String::from(
                    "Tuning specification contains a note not given in note frequency list",
                )));
            }
        }
        Ok(Tuning { values: map })
    }

    pub fn note(&self, string_idx: usize) -> Option<&Note> {
        self.values.get(&string_idx)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Note> {
        self.values.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tuning_empty() {
        let tuning_spec = vec![];
        let note_reg = NoteRegistry::from_notes(vec![]).unwrap();
        let tuning = Tuning::from_specification(&tuning_spec[..], &note_reg).unwrap();
        assert_eq!(None, tuning.iter().next());
    }

    #[test]
    fn test_tuning_empty_panic() {
        let tuning_spec = vec![];
        let note_reg = NoteRegistry::from_notes(vec![]).unwrap();
        let tuning = Tuning::from_specification(&tuning_spec[..], &note_reg).unwrap();
        assert_eq!(None, tuning.note(1));
    }

    #[test]
    fn test_tuning_nonempty_panic_wrong_idx() {
        let tuning_spec = vec![TuningSpecification {
            octave: 1,
            string: 1,
            name: NoteName::E,
        }];
        let note_reg = NoteRegistry::from_notes(vec![Note {
            name: NoteName::E,
            octave: 1,
            frequency: 53.5,
        }])
        .unwrap();
        let tuning = Tuning::from_specification(&tuning_spec[..], &note_reg).unwrap();
        assert_eq!(None, tuning.note(0));
    }

    #[test]
    fn test_tuning_nonempty_panic() {
        let tuning_spec = vec![TuningSpecification {
            octave: 1,
            string: 1,
            name: NoteName::E,
        }];
        let note_reg = NoteRegistry::from_notes(vec![Note {
            name: NoteName::E,
            octave: 1,
            frequency: 53.5,
        }])
        .unwrap();
        let tuning = Tuning::from_specification(&tuning_spec[..], &note_reg).unwrap();
        assert_eq!(None, tuning.note(2));
    }

    #[test]
    #[should_panic]
    fn test_tuning_incorrect_order() {
        let tuning_spec = vec![
            TuningSpecification {
                string: 1,
                octave: 4,
                name: NoteName::E,
            },
            TuningSpecification {
                string: 3,
                octave: 4,
                name: NoteName::E,
            },
            TuningSpecification {
                string: 2,
                octave: 4,
                name: NoteName::E,
            },
        ];
        let note_reg = NoteRegistry::from_notes(vec![
            Note {
                frequency: 35.5,
                octave: 4,
                name: NoteName::E,
            },
            Note {
                frequency: 35.5,
                octave: 4,
                name: NoteName::E,
            },
            Note {
                frequency: 35.5,
                octave: 4,
                name: NoteName::E,
            },
        ])
        .unwrap();
        Tuning::from_specification(&tuning_spec[..], &note_reg).unwrap();
    }

    #[test]
    fn test_tuning_note() {
        let tuning_spec = vec![
            TuningSpecification {
                string: 1,
                octave: 4,
                name: NoteName::E,
            },
            TuningSpecification {
                string: 2,
                octave: 4,
                name: NoteName::F,
            },
            TuningSpecification {
                string: 3,
                octave: 4,
                name: NoteName::G,
            },
        ];
        let note_vec = vec![
            Note {
                frequency: 35.5,
                octave: 4,
                name: NoteName::E,
            },
            Note {
                frequency: 36.0,
                octave: 4,
                name: NoteName::A,
            },
            Note {
                frequency: 36.5,
                octave: 4,
                name: NoteName::F,
            },
            Note {
                frequency: 37.0,
                octave: 4,
                name: NoteName::B,
            },
            Note {
                frequency: 37.5,
                octave: 4,
                name: NoteName::G,
            },
        ];
        let note_reg = NoteRegistry::from_notes(note_vec.clone()).unwrap();
        let tuning = Tuning::from_specification(&tuning_spec[..], &note_reg).unwrap();
        assert_eq!(&note_vec[0], tuning.note(1).unwrap());
        assert_eq!(&note_vec[2], tuning.note(2).unwrap());
        assert_eq!(&note_vec[4], tuning.note(3).unwrap());
    }

    #[test]
    fn test_tuning_iter() {
        let tuning_spec = vec![
            TuningSpecification {
                string: 1,
                octave: 4,
                name: NoteName::E,
            },
            TuningSpecification {
                string: 2,
                octave: 4,
                name: NoteName::F,
            },
            TuningSpecification {
                string: 3,
                octave: 4,
                name: NoteName::G,
            },
        ];
        let note_vec = vec![
            Note {
                frequency: 35.5,
                octave: 4,
                name: NoteName::E,
            },
            Note {
                frequency: 36.0,
                octave: 4,
                name: NoteName::A,
            },
            Note {
                frequency: 36.5,
                octave: 4,
                name: NoteName::F,
            },
            Note {
                frequency: 37.0,
                octave: 4,
                name: NoteName::B,
            },
            Note {
                frequency: 37.5,
                octave: 4,
                name: NoteName::G,
            },
        ];
        let note_reg = NoteRegistry::from_notes(note_vec.clone()).unwrap();
        let tuning = Tuning::from_specification(&tuning_spec[..], &note_reg).unwrap();
        let mut iter = tuning.iter();
        assert_eq!(Some(&note_vec[0]), iter.next());
        assert_eq!(Some(&note_vec[2]), iter.next());
        assert_eq!(Some(&note_vec[4]), iter.next());
        assert_eq!(None, iter.next());
    }
}
