mod audio_analysis;
mod game;
mod note;
mod visualization;

use crate::game::{FretRange, GameError, GameLogic, StringRange};
use crate::note::parse_freq_csv;

use cpal::Device;
use cpal::StreamConfig;

const GAME_TITLE: &str = "FRETBOARD TRAINER";

pub fn run(device: Device, config: StreamConfig, freq_csv_path: &str) -> Result<(), GameError> {
    let note_vec = parse_freq_csv(freq_csv_path)?;
    let mut game = GameLogic::new(
        device,
        config,
        String::from(GAME_TITLE),
        FretRange::new(0, 12),
        StringRange::new(1, 6 + 1),
        note_vec,
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
