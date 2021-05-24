use crate::fret_loc::FretLoc;
use crate::note::Note;

#[derive(Debug, Clone)]
pub struct GameState {
    pub target_note: Note,
    pub target_loc: FretLoc,
    pub needed_detection_count: usize,
    pub curr_detection_count: usize,
}
