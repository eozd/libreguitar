mod app;
mod audio_analysis;
mod game_logic;
mod game_state;
mod note;
mod visualization;

use crate::app::{App, AppError};
use crate::game_logic::{FretRange, StringRange};
use crate::note::{NoteRegistry, Tuning};

use cpal::Device;
use cpal::StreamConfig;

pub fn run(
    device: Device,
    config: StreamConfig,
    notes_csv_path: &str,
    tuning_csv_path: &str,
) -> Result<(), AppError> {
    let notes = NoteRegistry::from_csv(notes_csv_path)?;
    let tuning = Tuning::from_csv(tuning_csv_path, &notes)?;
    let mut app = App::new(
        device,
        config,
        FretRange::new(0, 12),
        StringRange::new(1, 6 + 1),
        notes,
        tuning,
    )?;
    app.run()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
