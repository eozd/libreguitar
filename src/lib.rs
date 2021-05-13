mod audio_analysis;
mod game;
mod note;

use crate::game::{FretRange, GameError, GameLogic, StringRange};
use crate::note::parse_freq_csv;

use cpal::traits::DeviceTrait;
use cpal::traits::StreamTrait;
use cpal::BuildStreamError;
use cpal::Device;
use cpal::Stream;
use cpal::StreamConfig;

const GAME_TITLE: &str = "FRETBOARD TRAINER";

pub fn run(device: Device, config: StreamConfig, freq_csv_path: &str) -> Result<(), GameError> {
    let note_vec = parse_freq_csv(freq_csv_path)?;
    let game = GameLogic::new(
        String::from(GAME_TITLE),
        FretRange::new(0, 12),
        StringRange::new(1, 6 + 1),
        config.sample_rate.0 as usize,
        note_vec,
    );

    let stream = build_stream(&device, &config, game)?;
    println!("Playing device...");
    stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(1000));
    Ok(())
}

fn build_stream(
    device: &Device,
    config: &StreamConfig,
    mut game: GameLogic,
) -> Result<Stream, BuildStreamError> {
    device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            game.tick(data);
        },
        move |_err| {
            // println!("Error reading data from device {}", _err);
        },
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
