
const GAME_TITLE: &str = "FRETBOARD TRAINER";
const MAX_FRETS: usize = 24;
const MAX_STRINGS: usize = 6;

struct FretRange {
    beg_fret: usize,
    end_fret: usize,
}

impl FretRange {
    fn new(beg_fret: usize, end_fret: usize) -> FretRange {
        assert!(
            beg_fret <= MAX_FRETS && end_fret <= MAX_FRETS + 1,
            "Maximum {} fret guitars are supported.",
            MAX_FRETS
        );
        assert!(
            beg_fret < end_fret,
            "Fret range must include at least one fret."
        );

        FretRange { beg_fret, end_fret }
    }
}

struct StringRange {
    beg_string: usize,
    end_string: usize,
}

impl StringRange {
    fn new(beg_string: usize, end_string: usize) -> StringRange {
        assert!(
            beg_string <= MAX_STRINGS && end_string <= MAX_STRINGS + 1,
            "Maximum {} string guitars are supported.",
            MAX_STRINGS
        );
        assert!(beg_string >= 1);
        assert!(
            beg_string < end_string,
            "String range must include at least one string."
        );

        StringRange {
            beg_string,
            end_string,
        }
    }
}

struct GameLogic {
    title: String,
    fret_range: FretRange,
    string_range: StringRange,
    frame_cnt: usize,
}

impl GameLogic {
    fn new(title: String, fret_range: FretRange, string_range: StringRange) -> GameLogic {
        GameLogic {
            title,
            fret_range,
            string_range,
            frame_cnt: 0,
        }
    }

    fn tick(&mut self, audio_data: &[f32]) {
        println!(
            "Tick {}: Maximum is {:?}",
            self.frame_cnt,
            audio_data.iter().cloned().fold(0. / 0., f32::max)
        );
        self.frame_cnt += 1;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
