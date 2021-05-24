use crate::core::csv::parse_csv;
use crate::core::{Note, NoteName};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct DuplicateNoteError(String);
impl fmt::Display for DuplicateNoteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DuplicateNoteError: {}", self.0)
    }
}

impl Error for DuplicateNoteError {}

pub struct NoteRegistry {
    notes: Vec<Note>,
}

impl NoteRegistry {
    pub fn from_csv(csv_path: &str) -> Result<NoteRegistry, Box<dyn Error>> {
        let notes = parse_csv(csv_path)?;
        match NoteRegistry::from_notes(notes) {
            Ok(v) => Ok(v),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn from_notes(mut notes: Vec<Note>) -> Result<NoteRegistry, DuplicateNoteError> {
        notes.sort_unstable_by(|a, b| a.frequency.partial_cmp(&b.frequency).unwrap());
        if !notes.is_empty() {
            for i in 0..notes.len() - 1 {
                if notes[i] == notes[i + 1] {
                    return Err(DuplicateNoteError(format!(
                        "Guitar frequency list has duplicates: {:?} and {:?}",
                        notes[i],
                        notes[i + 1]
                    )));
                }
            }
        }
        Ok(NoteRegistry { notes })
    }

    pub fn notes(&self) -> &Vec<Note> {
        &self.notes
    }

    fn find_note(&self, note: &Note) -> Option<usize> {
        self.notes.iter().position(|p| p == note)
    }

    pub fn get(&self, note_name: NoteName, octave: usize) -> Option<&Note> {
        let query_note = Note {
            name: note_name,
            octave,
            frequency: 0.0,
        };
        self.find_note(&query_note).map(|idx| &self.notes[idx])
    }

    pub fn iter_from<'a>(&'a self, starting_note: &'a Note) -> impl Iterator<Item = &'a Note> {
        if let Some(idx) = self.find_note(starting_note) {
            self.notes.iter().skip(idx)
        } else {
            self.notes.iter().skip(self.notes.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_registry_notes_empty() {
        let reg = NoteRegistry::from_notes(vec![]).unwrap();
        assert_eq!(0, reg.notes().len());
    }

    #[test]
    fn test_note_registry_get_empty() {
        let reg = NoteRegistry::from_notes(vec![]).unwrap();
        assert_eq!(None, reg.get(NoteName::A, 2));
        assert_eq!(None, reg.get(NoteName::E, 3));
        assert_eq!(None, reg.get(NoteName::F, 4));
        assert_eq!(None, reg.get(NoteName::GSharp, 1));
    }

    #[test]
    fn test_note_registry_iter_from_empty() {
        let note = Note {
            octave: 5,
            name: NoteName::E,
            frequency: 64.4,
        };
        let reg = NoteRegistry::from_notes(vec![]).unwrap();
        assert_eq!(None, reg.iter_from(&note).next());
    }

    #[test]
    fn test_note_registry_notes() {
        let reg = NoteRegistry::from_notes(vec![
            Note {
                octave: 5,
                name: NoteName::E,
                frequency: 300.0,
            },
            Note {
                octave: 3,
                name: NoteName::F,
                frequency: 62.2,
            },
            Note {
                octave: 4,
                name: NoteName::C,
                frequency: 75.5,
            },
        ])
        .unwrap();
        let expected = vec![
            Note {
                octave: 3,
                name: NoteName::F,
                frequency: 62.2,
            },
            Note {
                octave: 4,
                name: NoteName::C,
                frequency: 75.5,
            },
            Note {
                octave: 5,
                name: NoteName::E,
                frequency: 300.0,
            },
        ];
        assert_eq!(&expected, reg.notes());
    }

    #[test]
    fn test_note_registry_get() {
        let notes = vec![
            Note {
                octave: 3,
                name: NoteName::E,
                frequency: 300.0,
            },
            Note {
                octave: 3,
                name: NoteName::F,
                frequency: 62.2,
            },
            Note {
                octave: 3,
                name: NoteName::C,
                frequency: 75.5,
            },
        ];
        let reg = NoteRegistry::from_notes(notes.clone()).unwrap();
        assert_eq!(Some(&notes[0]), reg.get(NoteName::E, 3));
        assert_eq!(Some(&notes[1]), reg.get(NoteName::F, 3));
        assert_eq!(Some(&notes[2]), reg.get(NoteName::C, 3));
        assert_eq!(None, reg.get(NoteName::D, 2));
    }

    fn iter_len<T>(iter: impl Iterator<Item = T>) -> usize {
        let mut count = 0;
        for _ in iter {
            count += 1;
        }
        count
    }

    #[test]
    fn test_note_registry_iter_from() {
        let notes = vec![
            Note {
                octave: 3,
                name: NoteName::E,
                frequency: 300.0,
            },
            Note {
                octave: 3,
                name: NoteName::F,
                frequency: 62.2,
            },
            Note {
                octave: 3,
                name: NoteName::C,
                frequency: 75.5,
            },
        ];
        let nonexistent_note = Note {
            octave: 4,
            name: NoteName::D,
            frequency: 85.5,
        };
        let reg = NoteRegistry::from_notes(notes.clone()).unwrap();
        assert_eq!(1, iter_len(reg.iter_from(&notes[0])));
        assert_eq!(3, iter_len(reg.iter_from(&notes[1])));
        assert_eq!(2, iter_len(reg.iter_from(&notes[2])));
        assert_eq!(0, iter_len(reg.iter_from(&nonexistent_note)));
    }
}
