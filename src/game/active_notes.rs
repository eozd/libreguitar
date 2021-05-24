use crate::core::{FretLoc, FretRange, Note, NoteRegistry, StringRange, Tuning};
use std::collections::HashMap;

pub struct ActiveNotes {
    pub string_range: StringRange,
    pub fret_range: FretRange,
    notes: HashMap<FretLoc, Note>,
}

impl ActiveNotes {
    pub fn new(
        registry: &NoteRegistry,
        tuning: &Tuning,
        string_range: StringRange,
        fret_range: FretRange,
    ) -> ActiveNotes {
        let active_locs = active_locations(&string_range, &fret_range);
        let locs_and_notes = locs2notes(active_locs.into_iter(), tuning, registry);
        let mut notes = HashMap::new();
        for (loc, maybe_note) in locs_and_notes {
            if let Some(note) = maybe_note {
                notes.insert(loc, note.clone());
            } else {
                println!(
                    "Note on string {} fret {} does not exist in frequency list. Skipping...",
                    loc.string_idx, loc.fret_idx
                );
            }
        }

        ActiveNotes {
            string_range,
            fret_range,
            notes,
        }
    }

    pub fn get<'a>(&'a self, loc: &FretLoc) -> Option<&'a Note> {
        self.notes.get(loc)
    }
}

fn locs2notes<'a>(
    locs: impl Iterator<Item = FretLoc>,
    tuning: &'a Tuning,
    registry: &'a NoteRegistry,
) -> impl Iterator<Item = (FretLoc, Option<&'a Note>)> {
    locs.map(move |loc| {
        let open_string_note = tuning.note(loc.string_idx);
        if open_string_note.is_none() {
            return (loc, None);
        }
        let open_string_note = open_string_note.unwrap();
        let fret_idx = loc.fret_idx as i32;
        (loc, registry.add_semitones(open_string_note, fret_idx))
    })
}

fn active_locations(string_range: &StringRange, fret_range: &FretRange) -> Vec<FretLoc> {
    let mut out = Vec::new();
    for string_idx in string_range.r() {
        for fret_idx in fret_range.r() {
            out.push(FretLoc {
                string_idx,
                fret_idx,
            })
        }
    }
    out
}

#[cfg(test)]
mod active_note_tests {
    use super::*;
    use crate::core::{Note, NoteName, TuningSpecification};

    #[test]
    fn test_active_locations_one_cell() {
        let string_range = StringRange::new(1, 2);
        let fret_range = FretRange::new(3, 4);
        let active_locs = active_locations(&string_range, &fret_range);
        assert_eq!(1, active_locs.len());
        assert_eq!(
            FretLoc {
                string_idx: 1,
                fret_idx: 3
            },
            active_locs[0]
        );
    }

    #[test]
    fn test_active_locations_open_strings() {
        let string_range = StringRange::new(1, 4);
        let fret_range = FretRange::new(0, 1);
        let active_locs = active_locations(&string_range, &fret_range);
        assert_eq!(3, active_locs.len());
        assert_eq!(
            vec![
                FretLoc {
                    string_idx: 1,
                    fret_idx: 0
                },
                FretLoc {
                    string_idx: 2,
                    fret_idx: 0
                },
                FretLoc {
                    string_idx: 3,
                    fret_idx: 0
                },
            ],
            active_locs
        );
    }

    #[test]
    fn test_active_locations_fifth_pos() {
        let string_range = StringRange::new(1, 7);
        let fret_range = FretRange::new(5, 9);
        let active_locs = active_locations(&string_range, &fret_range);
        assert_eq!(24, active_locs.len());
        let mut expected = Vec::new();
        for string_idx in string_range.r() {
            for fret_idx in fret_range.r() {
                expected.push(FretLoc {
                    string_idx,
                    fret_idx,
                });
            }
        }
        assert_eq!(expected, active_locs);
    }

    #[test]
    fn test_locs2notes_empty() {
        let string_range = StringRange::new(1, 7);
        let fret_range = FretRange::new(5, 9);
        let active_locs = active_locations(&string_range, &fret_range);
        assert_eq!(24, active_locs.len());
        let mut expected = Vec::new();
        for string_idx in string_range.r() {
            for fret_idx in fret_range.r() {
                expected.push(FretLoc {
                    string_idx,
                    fret_idx,
                });
            }
        }
        assert_eq!(expected, active_locs);
    }

    #[test]
    fn test_locs2notes_unforgiven() {
        let registry = NoteRegistry::from_notes(vec![
            Note {
                octave: 3,
                name: NoteName::G,
                frequency: 205.0,
            },
            Note {
                octave: 3,
                name: NoteName::A,
                frequency: 220.0,
            },
            Note {
                octave: 3,
                name: NoteName::B,
                frequency: 246.0,
            },
            Note {
                octave: 4,
                name: NoteName::C,
                frequency: 261.0,
            },
        ])
        .unwrap();
        let tuning = Tuning::from_specification(
            &[TuningSpecification {
                name: NoteName::G,
                octave: 3,
                string: 3,
            }],
            &registry,
        )
        .unwrap();
        let locs = vec![
            FretLoc {
                string_idx: 3,
                fret_idx: 2,
            },
            FretLoc {
                string_idx: 3,
                fret_idx: 4,
            },
            FretLoc {
                string_idx: 3,
                fret_idx: 4,
            },
            FretLoc {
                string_idx: 3,
                fret_idx: 5,
            },
            FretLoc {
                string_idx: 3,
                fret_idx: 5,
            },
            FretLoc {
                string_idx: 3,
                fret_idx: 4,
            },
            FretLoc {
                string_idx: 3,
                fret_idx: 2,
            },
        ];
        let expected_notes = vec![
            registry.get(NoteName::A, 3).unwrap(),
            registry.get(NoteName::B, 3).unwrap(),
            registry.get(NoteName::B, 3).unwrap(),
            registry.get(NoteName::C, 4).unwrap(),
            registry.get(NoteName::C, 4).unwrap(),
            registry.get(NoteName::B, 3).unwrap(),
            registry.get(NoteName::A, 3).unwrap(),
        ];
        let actual_notes = locs2notes(locs.into_iter(), &tuning, &registry)
            .map(|(_, maybe_note)| maybe_note.unwrap())
            .collect::<Vec<&Note>>();
        assert_eq!(expected_notes, actual_notes);
    }

    #[test]
    fn test_active_notes_empty() {
        let registry = NoteRegistry::from_notes(vec![]).unwrap();
        let tuning = Tuning::from_specification(&[], &registry).unwrap();
        let active_notes = ActiveNotes::new(
            &registry,
            &tuning,
            StringRange::new(1, 7),
            FretRange::new(0, 12),
        );
        assert_eq!(0, active_notes.notes.len());
    }

    #[test]
    fn test_active_notes_fifth_position() {
        let notes = vec![
            Note {
                octave: 2,
                name: NoteName::E,
                frequency: 84.0,
            },
            Note {
                octave: 2,
                name: NoteName::A,
                frequency: 205.0,
            },
            Note {
                octave: 2,
                name: NoteName::B,
                frequency: 220.0,
            },
            Note {
                octave: 3,
                name: NoteName::C,
                frequency: 246.0,
            },
            Note {
                octave: 3,
                name: NoteName::D,
                frequency: 261.0,
            },
        ];
        let locs = vec![
            FretLoc {
                string_idx: 6,
                fret_idx: 0,
            },
            FretLoc {
                string_idx: 6,
                fret_idx: 5,
            },
            FretLoc {
                string_idx: 6,
                fret_idx: 7,
            },
            FretLoc {
                string_idx: 6,
                fret_idx: 8,
            },
            FretLoc {
                string_idx: 5,
                fret_idx: 5,
            },
        ];
        let registry = NoteRegistry::from_notes(notes.clone()).unwrap();
        let tuning = Tuning::from_specification(
            &[
                TuningSpecification {
                    name: NoteName::E,
                    octave: 2,
                    string: 6,
                },
                TuningSpecification {
                    name: NoteName::A,
                    octave: 2,
                    string: 5,
                },
            ],
            &registry,
        )
        .unwrap();
        let active_notes = ActiveNotes::new(
            &registry,
            &tuning,
            StringRange::new(1, 7),
            FretRange::new(0, 12),
        );
        for i in 0..notes.len() {
            assert_eq!(&notes[i], active_notes.get(&locs[i]).unwrap());
        }
    }
}
