use crate::core::NoteName;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Note {
    pub octave: i32,
    pub name: NoteName,
    pub frequency: f64,
}

impl Note {
    pub fn name_octave(&self) -> String {
        format!("{}{}", self.name, self.octave)
    }

    pub fn add_semitone(&self, semitones: i32) -> Note {
        let pos = pos_in_octave(self.name) as i32;
        let new_pos = pos + semitones;
        let octave_offset = new_pos / 12 - (new_pos < 0 && new_pos % 12 != 0) as i32;
        let octave = self.octave as i32 + octave_offset;
        let new_pos = new_pos.rem_euclid(12) as usize;
        let new_name = name_in_octave(new_pos);
        // TODO: Separate name-octave notes from frequencies since it is hard to
        // derive the frequency when doing these algebraic operations on notes.
        Note {
            octave,
            name: new_name,
            frequency: f64::NAN,
        }
    }
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.octave == other.octave && self.name == other.name
    }
}

impl Eq for Note {}

fn pos_in_octave(name: NoteName) -> usize {
    match name {
        NoteName::C => 0,
        NoteName::CSharp => 1,
        NoteName::D => 2,
        NoteName::DSharp => 3,
        NoteName::E => 4,
        NoteName::F => 5,
        NoteName::FSharp => 6,
        NoteName::G => 7,
        NoteName::GSharp => 8,
        NoteName::A => 9,
        NoteName::ASharp => 10,
        NoteName::B => 11,
    }
}

fn name_in_octave(pos: usize) -> NoteName {
    match pos {
        0 => NoteName::C,
        1 => NoteName::CSharp,
        2 => NoteName::D,
        3 => NoteName::DSharp,
        4 => NoteName::E,
        5 => NoteName::F,
        6 => NoteName::FSharp,
        7 => NoteName::G,
        8 => NoteName::GSharp,
        9 => NoteName::A,
        10 => NoteName::ASharp,
        11 => NoteName::B,
        _ => panic!("Octave position cannot be 12 or higher"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_equality() {
        let note_a4 = Note {
            octave: 4,
            name: NoteName::A,
            frequency: 440.0,
        };
        let note_a4_bad_freq = Note {
            octave: 4,
            name: NoteName::A,
            frequency: 445.0,
        };
        let note_a5 = Note {
            octave: 5,
            name: NoteName::A,
            frequency: 880.0,
        };
        let note_b4 = Note {
            octave: 5,
            name: NoteName::B,
            frequency: 493.9,
        };

        assert_eq!(
            note_a4,
            Note {
                octave: 4,
                name: NoteName::A,
                frequency: 440.0
            }
        );
        assert_eq!(note_a4, note_a4_bad_freq);
        assert_ne!(note_a4, note_a5);
        assert_ne!(note_a4, note_b4);
    }

    #[test]
    fn test_add_semitone_same_octave() {
        let note = Note {
            octave: 4,
            name: NoteName::A,
            frequency: 440.0,
        };
        assert_eq!(
            Note {
                octave: 4,
                name: NoteName::B,
                frequency: f64::NAN
            },
            note.add_semitone(2)
        );
        assert_eq!(
            Note {
                octave: 4,
                name: NoteName::G,
                frequency: f64::NAN
            },
            note.add_semitone(-2)
        );
        assert_eq!(
            Note {
                octave: 4,
                name: NoteName::FSharp,
                frequency: f64::NAN
            },
            note.add_semitone(-3)
        );
    }

    #[test]
    fn test_add_semitone_higher_octave() {
        let note = Note {
            octave: 3,
            name: NoteName::B,
            frequency: 440.0,
        };
        assert_eq!(
            Note {
                octave: 4,
                name: NoteName::CSharp,
                frequency: f64::NAN
            },
            note.add_semitone(2)
        );
        assert_eq!(
            Note {
                octave: 4,
                name: NoteName::ASharp,
                frequency: f64::NAN
            },
            note.add_semitone(11)
        );
        assert_eq!(
            Note {
                octave: 4,
                name: NoteName::B,
                frequency: f64::NAN
            },
            note.add_semitone(12)
        );
        assert_eq!(
            Note {
                octave: 6,
                name: NoteName::C,
                frequency: f64::NAN
            },
            note.add_semitone(25)
        );
    }

    #[test]
    fn test_add_semitone_lower_octave() {
        let note = Note {
            octave: 3,
            name: NoteName::CSharp,
            frequency: 440.0,
        };
        assert_eq!(
            Note {
                octave: 2,
                name: NoteName::B,
                frequency: f64::NAN
            },
            note.add_semitone(-2)
        );
        assert_eq!(
            Note {
                octave: 2,
                name: NoteName::D,
                frequency: f64::NAN
            },
            note.add_semitone(-11)
        );
        assert_eq!(
            Note {
                octave: 2,
                name: NoteName::CSharp,
                frequency: f64::NAN
            },
            note.add_semitone(-12)
        );
        assert_eq!(
            Note {
                octave: 1,
                name: NoteName::C,
                frequency: f64::NAN
            },
            note.add_semitone(-25)
        );
    }
}
