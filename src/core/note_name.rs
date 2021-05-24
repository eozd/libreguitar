use serde::Deserialize;
use std::fmt;

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
