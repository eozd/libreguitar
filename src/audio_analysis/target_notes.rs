use crate::note::Note;

pub struct TargetNotes {
    arr: Vec<Note>,
}

impl TargetNotes {
    pub fn new(mut arr: Vec<Note>) -> TargetNotes {
        assert!(!arr.is_empty(), "Target notes cannot be empty");
        arr.sort_unstable_by(|p, q| p.frequency.partial_cmp(&q.frequency).unwrap());
        TargetNotes { arr }
    }

    pub fn get_closest(&self, freq: f64) -> &Note {
        let search_result = self
            .arr
            .binary_search_by(|note| note.frequency.partial_cmp(&freq).unwrap());
        match search_result {
            Ok(idx) => &self.arr[idx],
            Err(idx) => {
                if idx == 0 {
                    &self.arr[idx]
                } else if idx == self.arr.len() {
                    &self.arr[idx - 1]
                } else {
                    let lower = &self.arr[idx - 1];
                    let upper = &self.arr[idx];
                    let lower_diff = freq - lower.frequency;
                    let upper_diff = upper.frequency - freq;
                    if lower_diff < upper_diff {
                        lower
                    } else {
                        upper
                    }
                }
            }
        }
    }

    pub fn resolution(&self) -> f64 {
        if self.arr.len() == 1 {
            0.0
        } else {
            self.arr[1].frequency - self.arr[0].frequency
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TargetNotes;
    use crate::note::{Note, NoteName};

    #[test]
    #[should_panic]
    fn test_empty_notes() {
        let notes = Vec::new();
        let _ = TargetNotes::new(notes);
    }

    #[test]
    fn test_resolution_single_note() {
        let notes = vec![Note {
            octave: 1,
            name: NoteName::A,
            frequency: 15.0,
        }];
        let target_notes = TargetNotes::new(notes);
        assert_eq!(0.0, target_notes.resolution());
    }

    #[test]
    fn test_resolution_general_case() {
        let notes = vec![
            Note {
                octave: 1,
                name: NoteName::A,
                frequency: 15.0,
            },
            Note {
                octave: 1,
                name: NoteName::B,
                frequency: 17.0,
            },
        ];
        let target_notes = TargetNotes::new(notes);
        assert_eq!(2.0, target_notes.resolution());
    }

    #[test]
    fn test_closest_note() {
        let notes = vec![
            Note {
                octave: 1,
                name: NoteName::A,
                frequency: 15.0,
            },
            Note {
                octave: 1,
                name: NoteName::B,
                frequency: 17.0,
            },
            Note {
                octave: 1,
                name: NoteName::B,
                frequency: 25.0,
            },
        ];
        let target_notes = TargetNotes::new(notes.clone());
        assert_eq!(&notes[0], target_notes.get_closest(-30.0));
        assert_eq!(&notes[0], target_notes.get_closest(3.0));
        assert_eq!(&notes[0], target_notes.get_closest(15.0));
        assert_eq!(&notes[0], target_notes.get_closest(15.6));

        assert_eq!(&notes[1], target_notes.get_closest(16.6));
        assert_eq!(&notes[1], target_notes.get_closest(17.0));
        assert_eq!(&notes[1], target_notes.get_closest(20.0));

        assert_eq!(&notes[2], target_notes.get_closest(23.0));
        assert_eq!(&notes[2], target_notes.get_closest(25.0));
        assert_eq!(&notes[2], target_notes.get_closest(500.0));
    }
}
