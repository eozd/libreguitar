use crate::core::NoteName;
use serde::Deserialize;

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
