mod audio_analysis;
mod game;
mod note;
mod visualization;

use crate::game::{FretRange, GameError, GameLogic, StringRange};
use crate::note::{NoteRegistry, Tuning};

use cpal::Device;
use cpal::StreamConfig;

const GAME_TITLE: &str = "FRETBOARD TRAINER";

pub fn run(
    device: Device,
    config: StreamConfig,
    notes_csv_path: &str,
    tuning_csv_path: &str,
) -> Result<(), GameError> {
    let notes = NoteRegistry::from_csv(notes_csv_path)?;
    let tuning = Tuning::from_csv(tuning_csv_path, &notes)?;
    let mut game = GameLogic::new(
        device,
        config,
        String::from(GAME_TITLE),
        FretRange::new(0, 24),
        StringRange::new(1, 6 + 1),
        notes,
        tuning,
    )?;
    game.run()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
