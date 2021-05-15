use crate::note::Note;

pub struct FrameData {
    pub note: Option<Note>,
    pub spectrogram: Vec<f64>,
}
