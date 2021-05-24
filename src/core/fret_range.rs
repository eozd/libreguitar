use std::ops::Range;

#[derive(Clone)]
pub struct FretRange {
    range: Range<usize>,
}

impl FretRange {
    pub fn new(beg_fret: usize, end_fret: usize) -> FretRange {
        assert!(
            beg_fret < end_fret,
            "Fret range must include at least one fret."
        );

        FretRange {
            range: beg_fret..end_fret,
        }
    }

    pub fn r(&self) -> Range<usize> {
        self.range.clone()
    }
}
