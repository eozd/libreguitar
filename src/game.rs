use crate::audio_analysis::AudioAnalyzer;
use crate::note::Note;
use crate::visualization::FrameData;
use crate::visualization::Visualizer;
use std::error::Error;
use std::sync::mpsc;
use thiserror::Error;

use cpal::traits::DeviceTrait;
use cpal::traits::StreamTrait;
use cpal::BufferSize;
use cpal::BuildStreamError;
use cpal::Device;
use cpal::Stream;
use cpal::StreamConfig;

const MAX_FRETS: usize = 24;
const MAX_STRINGS: usize = 6;

#[derive(Error, Debug)]
pub enum GameError {
    #[error(transparent)]
    BuildStreamError(#[from] cpal::BuildStreamError),
    #[error(transparent)]
    PlayStreamError(#[from] cpal::PlayStreamError),
    #[error(transparent)]
    UnknownError(#[from] Box<dyn Error>),
}

pub struct FretRange {
    beg_fret: usize,
    end_fret: usize,
}

impl FretRange {
    pub fn new(beg_fret: usize, end_fret: usize) -> FretRange {
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

pub struct StringRange {
    beg_string: usize,
    end_string: usize,
}

impl StringRange {
    pub fn new(beg_string: usize, end_string: usize) -> StringRange {
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

pub struct GameLogic {
    title: String,
    fret_range: FretRange,
    string_range: StringRange,
    audio_stream: Stream,
    visualizer: Visualizer,
}

impl GameLogic {
    pub fn new(
        device: Device,
        config: StreamConfig,
        title: String,
        fret_range: FretRange,
        string_range: StringRange,
        target_notes: Vec<Note>,
    ) -> Result<GameLogic, GameError> {
        let (analysis_tx, analysis_rx) = mpsc::channel();
        let analyzer = AudioAnalyzer::new(config.sample_rate.0 as usize, target_notes);
        let xaxis_props = (
            0.0,
            analyzer.n_bins() as f64 / analyzer.delta_f(),
            analyzer.delta_f(),
        );
        let audio_stream = build_audio_stream(device, config, analyzer, analysis_tx)?;
        let visualizer = Visualizer::new(analysis_rx, xaxis_props);
        Ok(GameLogic {
            title,
            fret_range,
            string_range,
            audio_stream,
            visualizer,
        })
    }

    pub fn run(&mut self) -> Result<(), GameError> {
        println!("Playing device...");
        // if let Some(note) = analysis.note {
        //     println!("Detected note: {:?}", note);
        // }
        self.audio_stream.play()?;
        self.visualizer.animate();
        Ok(())
    }
}

fn build_audio_stream<'a>(
    device: Device,
    config: StreamConfig,
    mut analyzer: AudioAnalyzer,
    tx: mpsc::Sender<FrameData>,
) -> Result<Stream, BuildStreamError> {
    let buffer_size = match config.buffer_size {
        BufferSize::Fixed(v) => Ok(v),
        BufferSize::Default => Err(BuildStreamError::InvalidArgument),
    }? as usize;
    let mut data_f64 = vec![0.0f64; buffer_size];
    device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            for i in 0..data.len() {
                data_f64[i] = data[i] as f64;
            }
            let analysis = analyzer.identify_note(&data_f64);
            let frame_data = FrameData {
                note: analysis.note,
                spectrogram: Vec::from(analysis.spectrogram),
            };
            tx.send(frame_data).unwrap();
            // TODO: send analysis results back to GameLogic
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
