use crate::audio_analysis::AudioAnalyzer;
use crate::note::{Note, NoteRegistry, Tuning};
use crate::visualization::{ConsoleData, ConsoleVisualizer, FrameData, GUIVisualizer};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::sync::mpsc;
use thiserror::Error;

use cpal::traits::DeviceTrait;
use cpal::traits::StreamTrait;
use cpal::BuildStreamError;
use cpal::Device;
use cpal::Stream;
use cpal::StreamConfig;

// TODO: get these from user
const MAX_FRETS: usize = 24;
const MAX_STRINGS: usize = 6;
const FRAME_RATE: f64 = 30.0;
const FRAME_PERIOD: f64 = 1.0 / FRAME_RATE;
// Increasing this value further would cause latency in real time frequency detection.
// Decreasing this value reduces FFT accuracy (particularly for low notes such as E2),
// as the low frequency notes don't get enough time to oscillate. The effect on
// high frequency notes such A4, A5, etc. is minimal even with block size of 128.
const MIN_N_SAMPLES_IN_AUDIO_BLOCK: usize = 2048;

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

#[derive(Debug)]
struct ConfigurationError(String);
impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ConfigurationError: {}", self.0)
    }
}
impl Error for ConfigurationError {}

struct ActiveNotes {
    notes: HashMap<(usize, usize), Note>,
}

impl ActiveNotes {
    fn new(
        registry: &NoteRegistry,
        tuning: &Tuning,
        string_range: &StringRange,
        fret_range: &FretRange,
    ) -> ActiveNotes {
        let mut notes = HashMap::new();
        for string_idx in string_range.beg_string..string_range.end_string {
            // TODO: read fret ranges while considering the tuning
            // TODO: read fret ranges while considering the tuning
            let open_note = tuning.note(string_idx);
            let mut note_iter = registry.iter_from(&open_note).skip(fret_range.beg_fret);
            for fret_idx in fret_range.beg_fret..fret_range.end_fret {
                match note_iter.next() {
                    Some(curr_note) => {
                        notes.insert((string_idx, fret_idx), curr_note.clone());
                    }
                    None => {
                        // TODO: use logging library
                        println!("Note on string {} fret {} does not exist in frequency list. Skipping...", string_idx, fret_idx);
                    }
                }
            }
        }

        ActiveNotes { notes }
    }
}

pub struct GameLogic {
    title: String,
    fret_range: FretRange,
    string_range: StringRange,
    note_registry: NoteRegistry,
    tuning: Tuning,
    active_notes: ActiveNotes,
    audio_stream: Stream,
    gui_visualizer: GUIVisualizer,
    console_visualizer: ConsoleVisualizer,
}

impl GameLogic {
    pub fn new(
        device: Device,
        config: StreamConfig,
        title: String,
        fret_range: FretRange,
        string_range: StringRange,
        note_registry: NoteRegistry,
        tuning: Tuning,
    ) -> Result<GameLogic, GameError> {
        let analyzer = AudioAnalyzer::new(config.sample_rate.0 as usize, note_registry.notes());
        let xaxis_props = (
            0.0,
            analyzer.n_bins() as f64 / analyzer.delta_f(),
            analyzer.delta_f(),
        );
        let (gui_tx, gui_rx) = mpsc::channel();
        let (console_tx, console_rx) = mpsc::channel();
        let gui_visualizer = GUIVisualizer::new(gui_rx, xaxis_props);
        let console_visualizer = ConsoleVisualizer::new(console_rx);
        let audio_stream = build_data_channels(device, config, analyzer, gui_tx, console_tx)?;
        let active_notes = ActiveNotes::new(&note_registry, &tuning, &string_range, &fret_range);
        Ok(GameLogic {
            title,
            fret_range,
            string_range,
            note_registry,
            tuning,
            active_notes,
            audio_stream,
            gui_visualizer,
            console_visualizer,
        })
    }

    fn is_running(&self) -> bool {
        self.gui_visualizer.is_open() && self.console_visualizer.is_open()
    }

    pub fn run(&mut self) -> Result<(), GameError> {
        self.audio_stream.play()?;
        while self.is_running() {
            self.console_visualizer.draw();
            self.gui_visualizer.draw();
            std::thread::sleep(std::time::Duration::from_secs_f64(FRAME_PERIOD));
        }
        Ok(())
    }
}

fn build_data_channels(
    device: Device,
    config: StreamConfig,
    mut analyzer: AudioAnalyzer,
    gui_tx: mpsc::Sender<FrameData>,
    console_tx: mpsc::Sender<ConsoleData>,
) -> Result<Stream, BuildStreamError> {
    let mut audio_buffer = VecDeque::from(vec![0.0f64; MIN_N_SAMPLES_IN_AUDIO_BLOCK]);
    audio_buffer.shrink_to_fit();
    let n_channels = config.channels as usize;
    // TODO: get from user
    let listened_channel = 0;
    device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            read_channel_buffered(data, n_channels, listened_channel, &mut audio_buffer);
            let analysis = analyzer.identify_note(audio_buffer.iter().cloned());
            let console_data = ConsoleData {
                note: analysis.note.clone(),
            };
            console_tx.send(console_data).unwrap();
            let frame_data = FrameData {
                note: analysis.note,
                spectrogram: Vec::from(analysis.spectrogram),
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
