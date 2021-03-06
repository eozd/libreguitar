use crate::audio_analysis::AnalysisResult;
use crate::core::{FretLoc, FretRange, GameCfg, Note, NoteRegistry, StringRange, Tuning};
use crate::game::{ActiveNotes, GameState};
use std::error::Error;
use std::fmt;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub struct GameError(String);

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GameError: {}", self.0)
    }
}

impl Error for GameError {}

enum ThreadCtrl {
    Start,
    // Stop,
}

pub struct GameLogic {
    ctrl_tx: mpsc::Sender<ThreadCtrl>,
    fret_range: FretRange,
    string_range: StringRange,
}

fn wait_until_start(rx: &mpsc::Receiver<ThreadCtrl>) -> Result<(), mpsc::RecvError> {
    loop {
        let res = rx.recv();
        if let Ok(ThreadCtrl::Start) = res {
            return Ok(());
        } else if let Err(err) = res {
            return Err(err);
        }
    }
}

impl GameLogic {
    pub fn new(
        rx: mpsc::Receiver<AnalysisResult>,
        tx_vec: Vec<mpsc::Sender<GameState>>,
        note_registry: NoteRegistry,
        tuning: Tuning,
        config: GameCfg,
    ) -> GameLogic {
        let fret_range = FretRange::new(config.fret_range.0, config.fret_range.1);
        let string_range = StringRange::new(config.string_range.0, config.string_range.1);
        let active_notes = ActiveNotes::new(
            &note_registry,
            &tuning,
            string_range.clone(),
            fret_range.clone(),
        );
        let (ctrl_tx, ctrl_rx) = mpsc::channel();
        let needed_detection_count = config.note_count_for_acceptance;
        thread::spawn(move || {
            wait_until_start(&ctrl_rx).unwrap();
            let mut rng = rand::thread_rng();
            loop {
                // if let Ok(ThreadCtrl::Stop) = ctrl_rx.try_recv() {
                //     wait_until_start(&ctrl_rx).unwrap();
                // }
                let (target_note, target_loc) = pick_note(&active_notes, &mut rng);
                let mut state = GameState {
                    target_note: target_note.clone(),
                    target_loc,
                    needed_detection_count,
                    curr_detection_count: 0,
                };
                for tx in tx_vec.iter() {
                    tx.send(state.clone()).unwrap();
                }
                for analysis in rx.iter() {
                    if let Some(note) = analysis.note {
                        state.curr_detection_count += (note == state.target_note) as usize;
                    }
                    if state.curr_detection_count > 0
                        && state.curr_detection_count % config.state_update_period == 0
                    {
                        for tx in tx_vec.iter() {
                            tx.send(state.clone()).unwrap();
                        }
                    }
                    if state.curr_detection_count == needed_detection_count {
                        break;
                    }
                }
            }
        });
        GameLogic {
            ctrl_tx,
            fret_range,
            string_range,
        }
    }

    pub fn fret_range(&self) -> &FretRange {
        &self.fret_range
    }

    pub fn string_range(&self) -> &StringRange {
        &self.string_range
    }

    pub fn play(&mut self) -> Result<(), GameError> {
        self.ctrl_tx
            .send(ThreadCtrl::Start)
            .map_err(|_| GameError(String::from("Could not start thread")))
    }

    // pub fn pause(&mut self) -> Result<(), GameError> {
    //     self.ctrl_tx
    //         .send(ThreadCtrl::Stop)
    //         .map_err(|_| GameError(String::from("Could not stop thread")))
    // }
}

fn pick_note<'a>(notes: &'a ActiveNotes, rng: &mut impl rand::Rng) -> (&'a Note, FretLoc) {
    let string_idx = rng.gen_range(notes.string_range.r());
    let fret_idx = rng.gen_range(notes.fret_range.r());
    let key = FretLoc {
        string_idx,
        fret_idx,
    };
    (notes.get(&key).unwrap(), key)
}

#[derive(Debug)]
struct ConfigurationError(String);
impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ConfigurationError: {}", self.0)
    }
}
impl Error for ConfigurationError {}

#[cfg(test)]
mod game_logic_tests {
    use super::*;
    #[test]
    fn test_equality() {}
}
