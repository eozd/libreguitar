use std::ops::Range;

#[derive(Clone)]
pub struct StringRange {
    range: Range<usize>,
}

impl StringRange {
    pub fn new(beg_string: usize, end_string: usize) -> StringRange {
        assert!(beg_string >= 1);
        assert!(
            beg_string < end_string,
            "String range must include at least one string."
        );

        StringRange {
            range: beg_string..end_string,
        }
    }

    pub fn r(&self) -> Range<usize> {
        self.range.clone()
    }
}
