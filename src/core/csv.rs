use serde::de::DeserializeOwned;
use std::error::Error;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Note, NoteName};
    use csv::Reader;
    use serde::Deserialize;

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

    #[derive(PartialEq, Deserialize)]
    struct TestTuneSpec {
        string: usize,
        octave: usize,
        name: NoteName,
    }

    #[test]
    fn parse_tuning_csv() {
        let data = "string,octave,name\n\
                    1,2,C\n\
                    2,3,A\n";
        let expected = vec![
            TestTuneSpec {
                string: 1,
                octave: 2,
                name: NoteName::C,
            },
            TestTuneSpec {
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
