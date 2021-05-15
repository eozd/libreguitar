use crate::note::Note;

pub struct AnalysisResult<'a> {
    pub note: Option<Note>,
    pub spectrogram: &'a [f64],
}
