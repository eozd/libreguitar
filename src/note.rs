use serde::Deserialize;
use std::error::Error;

fn parse_freq_csv_impl<R>(mut rdr: csv::Reader<R>) -> Result<Vec<Note>, Box<dyn Error>>
where
    R: std::io::Read,
{
    let mut iter = rdr.deserialize();
    let mut out = Vec::new();
    while let Some(result) = iter.next() {
        out.push(result?);
    }
    Ok(out)
}

pub fn parse_freq_csv(csv_path: &str) -> Result<Vec<Note>, Box<dyn Error>> {
    let rdr = csv::Reader::from_path(csv_path)?;
    parse_freq_csv_impl(rdr)
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
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

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Note {
    pub octave: usize,
    pub name: NoteName,
    pub frequency: f64,
}

#[cfg(test)]
mod tests {
    use super::parse_freq_csv_impl;
    use crate::note::{Note, NoteName};
    use csv::Reader;

    #[test]
    fn parse_empty_csv() {
        let data = "octave,name,frequency";
        let rdr = Reader::from_reader(data.as_bytes());
        let expected: Vec<Note> = Vec::new();
        let actual = parse_freq_csv_impl(rdr).unwrap();
        assert!(expected.into_iter().eq(actual.into_iter()));
    }

    #[test]
    #[should_panic]
    fn parse_invalid_csv() {
        let data = "octave,name,frequency\n\
                    2.0,C,31.23\n";
        let rdr = Reader::from_reader(data.as_bytes());
        for result in parse_freq_csv_impl(rdr).unwrap().iter() {
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
        let actual = parse_freq_csv_impl(rdr).unwrap();
        assert!(expected.into_iter().eq(actual.into_iter()));
    }
}
