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
