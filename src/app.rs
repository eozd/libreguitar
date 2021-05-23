use crate::audio_analysis::{AnalysisResult, AudioAnalyzer};
use crate::game_logic::{GameError, GameLogic};
use crate::note::{NoteRegistry, Tuning};
use crate::visualization::{ConsoleVisualizer, FrameData, GUIVisualizer};
use crate::{AppCfg, Cfg};
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
    gui_visualizer: GUIVisualizer,
    console_visualizer: ConsoleVisualizer,
    game_logic: GameLogic,
    app_cfg: AppCfg,
}

impl App {
    pub fn new(device: Device, device_config: StreamConfig, cfg: Cfg) -> Result<App, AppError> {
        let app_cfg = cfg.app;
        let note_registry = NoteRegistry::from_csv(&app_cfg.frequencies_path)?;
        let tuning = Tuning::from_csv(&app_cfg.tuning_path, &note_registry)?;
        let analyzer = AudioAnalyzer::new(
            device_config.sample_rate.0 as usize,
            note_registry.notes(),
            cfg.audio,
        );
        let xaxis_props = (
            0.0,
            analyzer.n_bins() as f64 / analyzer.delta_f(),
            analyzer.delta_f(),
        );
        let (gui_tx, gui_rx) = mpsc::channel();
        let (analysis_tx, analysis_rx) = mpsc::channel();
        let (console_tx, console_rx) = mpsc::channel();
        let gui_visualizer = GUIVisualizer::new(gui_rx, xaxis_props, cfg.gui);
        let console_visualizer = ConsoleVisualizer::new(console_rx);
        let audio_stream = build_connection_protocols(
            device,
            device_config,
            analyzer,
            gui_tx,
            analysis_tx,
            app_cfg.block_size,
        )?;
        let game_logic = GameLogic::new(
            analysis_rx,
            vec![console_tx],
            note_registry,
            tuning,
            cfg.game,
        );
        Ok(App {
            audio_stream,
            gui_visualizer,
            console_visualizer,
            game_logic,
            app_cfg,
        })
    }

    fn is_running(&self) -> bool {
        self.gui_visualizer.is_open() && self.console_visualizer.is_open()
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        self.audio_stream.play()?;
        self.game_logic.play()?;
        let frame_period = 1.0 / self.app_cfg.fps;
        while self.is_running() {
            self.console_visualizer.draw();
            self.gui_visualizer.draw();
            std::thread::sleep(std::time::Duration::from_secs_f64(frame_period));
        }
        Ok(())
    }
}

fn build_connection_protocols(
    device: Device,
    device_config: StreamConfig,
    mut analyzer: AudioAnalyzer,
    gui_tx: mpsc::Sender<FrameData>,
    analysis_tx: mpsc::Sender<AnalysisResult>,
    block_size: usize,
) -> Result<Stream, BuildStreamError> {
    let mut audio_buffer = VecDeque::from(vec![0.0f64; block_size]);
    audio_buffer.shrink_to_fit();
    let n_channels = device_config.channels as usize;
    // TODO: get from user
    let listened_channel = 0;
    device.build_input_stream(
        &device_config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            read_channel_buffered(data, n_channels, listened_channel, &mut audio_buffer);
            let analysis = analyzer.identify_note(audio_buffer.iter().cloned());
            // send data to game logic
            analysis_tx.send(analysis).unwrap();
            // send data to GUI
            let frame_data = FrameData {
                spectrogram: analyzer.spectrogram().clone(),
            };
            gui_tx.send(frame_data).unwrap();
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
