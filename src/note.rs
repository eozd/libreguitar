use serde::Deserialize;
use std::error::Error;

pub fn parse_freq_csv(csv_path: &str) -> Result<Vec<Note>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(csv_path)?;
    let mut iter = rdr.deserialize();
    let mut out = Vec::new();
    while let Some(result) = iter.next() {
        out.push(result?);
    }
    Ok(out)
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct Note {
    pub octave: usize,
    pub name: NoteName,
    pub frequency: f32,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
