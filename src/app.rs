use crate::audio_analysis::AudioAnalyzer;
use crate::core::{Cfg, NoteRegistry, Tuning};
use crate::game::{GameError, GameLogic};
use crate::visualization::{ConsoleVisualizer, Visualizer};
#[cfg(feature = "gui")]
use crate::visualization::{FrameData, GUIVisualizer, GuiCfg};
use std::collections::VecDeque;
use std::error::Error;
use std::sync::mpsc;
use thiserror::Error;

use cpal::traits::DeviceTrait;
use cpal::traits::StreamTrait;
use cpal::BuildStreamError;
use cpal::Device;
use cpal::Stream;
use cpal::StreamConfig;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    BuildStreamError(#[from] cpal::BuildStreamError),
    #[error(transparent)]
    PlayStreamError(#[from] cpal::PlayStreamError),
    #[error(transparent)]
    GameError(#[from] GameError),
    #[error(transparent)]
    UnknownError(#[from] Box<dyn Error>),
}

pub struct App {
    audio_stream: Stream,
    visualizers: Vec<Box<dyn Visualizer>>,
    game_logic: GameLogic,
    frame_period: f64,
}

impl App {
    pub fn new(device: Device, device_config: StreamConfig, cfg: Cfg) -> Result<App, AppError> {
        let app_cfg = cfg.app;
        let note_registry = NoteRegistry::from_csv(&app_cfg.frequencies_path)?;
        let tuning = Tuning::from_csv(&app_cfg.tuning_path, &note_registry)?;
        let mut analyzer = AudioAnalyzer::new(
            device_config.sample_rate.0 as usize,
            note_registry.notes(),
            cfg.audio,
        );
        let (analysis_tx, analysis_rx) = mpsc::channel();
        let (console_tx, console_rx) = mpsc::channel();
        let game_logic = GameLogic::new(
            analysis_rx,
            vec![console_tx],
            note_registry,
            tuning.clone(),
            cfg.game,
        );
        let console_visualizer = ConsoleVisualizer::new(
            console_rx,
            game_logic.fret_range().clone(),
            game_logic.string_range().clone(),
            cfg.console,
            tuning,
        );
        let visualizers: Vec<Box<dyn Visualizer>> = vec![Box::new(console_visualizer)];
        #[cfg(feature = "gui")]
        let (gui_tx, gui_rx) = mpsc::channel();
        #[cfg(feature = "gui")]
        let visualizers = add_gui_visualizer(
            visualizers,
            analyzer.n_bins(),
            analyzer.delta_f(),
            gui_rx,
            cfg.gui,
        );
        let audio_read_callback: Box<CallbackFn> =
            Box::new(move |data: Box<dyn ExactSizeIterator<Item = f64>>| {
                let analysis = analyzer.identify_note(data);
                // send data to game logic
                analysis_tx.send(analysis).unwrap();
                #[cfg(feature = "gui")]
                {
                    // send data to GUI
                    let frame_data = FrameData {
                        spectrogram: analyzer.spectrogram().clone(),
                    };
                    gui_tx.send(frame_data).unwrap();
                }
            });
        let audio_stream = create_audio_stream(
            device,
            device_config,
            app_cfg.block_size,
            audio_read_callback,
        )?;
        Ok(App {
            audio_stream,
            visualizers,
            game_logic,
            frame_period: 1.0 / app_cfg.fps,
        })
    }

    fn is_running(&self) -> bool {
        self.visualizers.iter().all(|v| v.is_open())
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        self.audio_stream.play()?;
        self.game_logic.play()?;
        while self.is_running() {
            for visualizer in self.visualizers.iter_mut() {
                visualizer.draw();
            }
            std::thread::sleep(std::time::Duration::from_secs_f64(self.frame_period));
        }
        Ok(())
    }
}

#[cfg(feature = "gui")]
fn add_gui_visualizer(
    mut visualizers: Vec<Box<dyn Visualizer>>,
    n_bins: usize,
    delta_f: f64,
    gui_rx: mpsc::Receiver<FrameData>,
    cfg: GuiCfg,
) -> Vec<Box<dyn Visualizer>> {
    let xaxis_props = (0.0, n_bins as f64 / delta_f, delta_f);
    let gui_visualizer = GUIVisualizer::new(gui_rx, xaxis_props, cfg);
    visualizers.push(Box::new(gui_visualizer));
    visualizers
}

type CallbackFn = dyn for<'a> FnMut(Box<dyn ExactSizeIterator<Item = f64> + 'a>) + Send;

fn create_audio_stream(
    device: Device,
    device_config: StreamConfig,
    block_size: usize,
    mut callback: Box<CallbackFn>,
) -> Result<Stream, BuildStreamError> {
    let mut audio_buffer = VecDeque::from(vec![0.0f64; block_size]);
    audio_buffer.shrink_to_fit();
    let n_channels = device_config.channels as usize;
    // TODO: get from user
    let listened_channel = 1;
    device.build_input_stream(
        &device_config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            read_channel_buffered(data, n_channels, listened_channel, &mut audio_buffer);
            callback(Box::new(audio_buffer.iter().cloned()));
        },
        move |_err| {
            // Mainly happens if we miss some audio frames.
            // println!("Error reading data from device {}", _err);
        },
    )
}

fn read_channel_buffered(
    data: &[f32],
    n_channels: usize,
    channel: usize,
    buffer: &mut VecDeque<f64>,
) {
    let channel_indices = (channel..data.len()).step_by(n_channels);
    let n_new_values = channel_indices.len();
    if n_new_values >= buffer.len() {
        buffer.clear();
    } else {
        for _ in 0..n_new_values {
            buffer.pop_front();
        }
    }
    for i in channel_indices {
        buffer.push_back(data[i] as f64);
    }
}

#[cfg(test)]
mod game_tests {
    use super::*;
    #[test]
    fn read_channel_buffered_empty_buffer_empty_data() {
        let mut buffer = VecDeque::new();
        let data = Vec::new();
        read_channel_buffered(&data, 2, 0, &mut buffer);
        assert_eq!(0, buffer.len());
    }

    #[test]
    fn read_channel_buffered_empty_data() {
        let mut buffer = VecDeque::from(vec![1.0f64; 64]);
        let expected = buffer.clone();
        let data = Vec::new();
        read_channel_buffered(&data, 3, 1, &mut buffer);
        assert_eq!(expected, buffer);
    }

    #[test]
    fn read_channel_buffered_empty_buffer() {
        let mut buffer = VecDeque::new();
        let data: Vec<f32> = (0..100).map(|x| x as f32).collect();
        let expected: VecDeque<f64> = data.iter().cloned().step_by(2).map(|x| x as f64).collect();
        read_channel_buffered(&data, 2, 0, &mut buffer);
        assert_eq!(expected, buffer);
    }

    #[test]
    fn read_channel_buffered_less_data_than_buffer() {
        let mut buffer = VecDeque::from(vec![5000.0f64; 200]);
        let data: Vec<f32> = (0..100).map(|x| x as f32).collect();
        let expected: VecDeque<f64> = buffer
            .iter()
            .cloned()
            .skip(50)
            .chain(data.iter().cloned().step_by(2).map(|x| x as f64))
            .collect();
        read_channel_buffered(&data, 2, 0, &mut buffer);
        assert_eq!(expected, buffer);
    }

    #[test]
    fn read_channel_buffered_same_data_as_buffer() {
        let mut buffer = VecDeque::from(vec![5000.0f64; 200]);
        let data: Vec<f32> = (0..200).map(|x| x as f32).collect();
        let expected: VecDeque<f64> = data.iter().cloned().map(|x| x as f64).collect();
        read_channel_buffered(&data, 1, 0, &mut buffer);
        assert_eq!(expected, buffer);
    }

    #[test]
    fn read_channel_buffered_more_data_than_buffer() {
        let mut buffer = VecDeque::from(vec![5000.0f64; 50]);
        let data: Vec<f32> = (0..200).map(|x| x as f32).collect();
        let expected: VecDeque<f64> = data.iter().cloned().map(|x| x as f64).collect();
        read_channel_buffered(&data, 1, 0, &mut buffer);
        assert_eq!(expected, buffer);
    }
}
