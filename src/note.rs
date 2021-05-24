use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

fn parse_csv_impl<R, T>(mut rdr: csv::Reader<R>) -> Result<Vec<T>, Box<dyn Error>>
where
    R: std::io::Read,
    T: DeserializeOwned,
{
    let iter = rdr.deserialize();
    let mut out = Vec::new();
    for result in iter {
        out.push(result?);
    }
    Ok(out)
}

pub fn parse_csv<T>(csv_path: &str) -> Result<Vec<T>, Box<dyn Error>>
where
    T: DeserializeOwned,
{
    let rdr = csv::Reader::from_path(csv_path)?;
    parse_csv_impl(rdr)
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum NoteName {
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

impl fmt::Display for NoteName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            NoteName::A => "A",
            NoteName::ASharp => "A♯",
            NoteName::B => "B",
            NoteName::C => "C",
            NoteName::CSharp => "C♯",
            NoteName::D => "D",
            NoteName::DSharp => "D♯",
            NoteName::E => "E",
            NoteName::F => "F",
            NoteName::FSharp => "F♯",
            NoteName::G => "G",
            NoteName::GSharp => "G♯",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Note {
    pub octave: usize,
    pub name: NoteName,
    pub frequency: f64,
}

impl Note {
    pub fn name_octave(&self) -> String {
        format!("{}{}", self.name, self.octave)
    }
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.octave == other.octave && self.name == other.name
    }
}

impl Eq for Note {}

impl PartialOrd for Note {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Note {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.octave == other.octave {
            return self.name.cmp(&other.name);
        }
        self.octave.cmp(&other.octave)
    }
}

pub struct NoteRegistry {
    notes: Vec<Note>,
}

#[derive(Debug)]
struct DuplicateNoteError(String);
impl fmt::Display for DuplicateNoteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DuplicateNoteError: {}", self.0)
    }
}
impl Error for DuplicateNoteError {}

#[derive(Debug)]
struct InvalidTuningError(String);
impl fmt::Display for InvalidTuningError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InvalidTuningError: {}", self.0)
    }
}
impl Error for InvalidTuningError {}

impl NoteRegistry {
    pub fn from_csv(csv_path: &str) -> Result<NoteRegistry, Box<dyn Error>> {
        let mut notes = parse_csv(csv_path)?;
        notes.sort_unstable();
        for i in 0..notes.len() - 1 {
            if notes[i] == notes[i + 1] {
                return Err(Box::new(DuplicateNoteError(format!(
                    "Guitar frequency list has duplicates: {:?} and {:?}",
                    notes[i],
                    notes[i + 1]
                ))));
            }
        }
        Ok(NoteRegistry { notes })
    }

    pub fn notes(&self) -> &Vec<Note> {
        &self.notes
    }

    pub fn get(&self, note_name: NoteName, octave: usize) -> Option<&Note> {
        let query_note = Note {
            name: note_name,
            octave,
            frequency: 0.0,
        };
        if let Ok(idx) = self.notes.binary_search(&query_note) {
            Some(&self.notes[idx])
        } else {
            None
        }
    }

    pub fn iter_from<'a>(&'a self, starting_note: &'a Note) -> impl Iterator<Item = &'a Note> {
        if let Ok(idx) = self.notes.binary_search(starting_note) {
            self.notes.iter().skip(idx)
        } else {
            self.notes.iter().skip(self.notes.len())
        }
    }
}

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
        let mut map = Vec::new();
        for (i, row) in tuning_spec.iter().enumerate() {
            if row.string - 1 != i {
                return Err(Box::new(InvalidTuningError(String::from(
                    "Tuning specification needs strings to be numbered as 1, 2, 3, ...",
                ))));
            }
            if let Some(note) = note_registry.get(row.name, row.octave) {
                map.push(note.clone());
            } else {
                return Err(Box::new(InvalidTuningError(String::from(
                    "Tuning specification contains a note not given in note frequency list",
                ))));
            }
        }
        if map.is_empty() {
            return Err(Box::new(InvalidTuningError(String::from(
                "Tuning specification needs at least one string",
            ))));
        }
        Ok(Tuning { values: map })
    }

    pub fn note(&self, string_idx: usize) -> &Note {
        assert!(
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

#[cfg(test)]
mod tests {
    use super::parse_csv_impl;
    use super::TuningSpecification;
    use crate::note::{Note, NoteName};
    use csv::Reader;

    #[test]
    fn parse_empty_csv() {
        let data = "octave,name,frequency";
        let rdr = Reader::from_reader(data.as_bytes());
        let expected: Vec<Note> = Vec::new();
        let actual = parse_csv_impl(rdr).unwrap();
        assert!(expected.into_iter().eq(actual.into_iter()));
    }

    #[test]
    #[should_panic]
    fn parse_invalid_csv() {
        let data = "octave,name,frequency\n\
                    2.0,C,31.23\n";
        let rdr = Reader::from_reader(data.as_bytes());
        let results: Vec<Note> = parse_csv_impl(rdr).unwrap();
        for result in results.iter() {
            let _ = result;
        }
    }

    #[test]
    fn parse_valid_csv() {
        let data = "octave,name,frequency\n\
                    2,C,31.23\n\
                    2,D,32.23\n\
                    4,A,65.23\n";
        let expected = vec![
            Note {
                octave: 2,
                name: NoteName::C,
                frequency: 31.23,
            },
            Note {
                octave: 2,
                name: NoteName::D,
                frequency: 32.23,
            },
            Note {
                octave: 4,
                name: NoteName::A,
                frequency: 65.23,
            },
        ];
        let rdr = Reader::from_reader(data.as_bytes());
        let actual = parse_csv_impl(rdr).unwrap();
        assert!(expected.into_iter().eq(actual.into_iter()));
    }

    #[test]
    fn parse_tuning_csv() {
        let data = "string,octave,name\n\
                    1,2,C\n\
                    2,3,A\n";
        let expected = vec![
            TuningSpecification {
                string: 1,
                octave: 2,
                name: NoteName::C,
            },
            TuningSpecification {
                string: 2,
                octave: 3,
                name: NoteName::A,
            },
        ];
        let rdr = Reader::from_reader(data.as_bytes());
        let actual = parse_csv_impl(rdr).unwrap();
        assert!(expected.into_iter().eq(actual.into_iter()));
    }
}
