use crate::audio_analysis::AnalysisResult;
use crate::game_state::GameState;
use crate::note::{Note, NoteRegistry, Tuning};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::ops::Range;
use std::sync::mpsc;
use std::thread;

// TODO: get these from user
const MAX_FRETS: usize = 24;
const MAX_STRINGS: usize = 6;

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
        string_range: StringRange,
        fret_range: FretRange,
    ) -> GameLogic {
        let active_notes = ActiveNotes::new(&note_registry, &tuning, string_range, fret_range);
        let (ctrl_tx, ctrl_rx) = mpsc::channel();
        thread::spawn(move || {
            wait_until_start(&ctrl_rx).unwrap();
            let mut rng = rand::thread_rng();
            loop {
                // if let Ok(ThreadCtrl::Stop) = ctrl_rx.try_recv() {
                //     wait_until_start(&ctrl_rx).unwrap();
                // }
                let target_note = pick_note(&active_notes, &mut rng);
                let state = GameState {
                    target_note: target_note.clone(),
                };
                for tx in tx_vec.iter() {
                    tx.send(state.clone()).unwrap();
                }
                let mut seen_count = 0;
                for analysis in rx.iter() {
                    if let Some(note) = analysis.note {
                        seen_count += (note == state.target_note) as usize;
                    }
                    if seen_count == 50 {
                        break;
                    }
                }
            }
        });
        GameLogic { ctrl_tx }
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

fn pick_note<'a>(notes: &'a ActiveNotes, rng: &mut impl rand::Rng) -> &'a Note {
    let string_idx = rng.gen_range(notes.string_range.range());
    let fret_idx = rng.gen_range(notes.fret_range.range());
    let key = (string_idx, fret_idx);
    notes.notes.get(&key).unwrap()
}

pub struct FretRange {
    range: Range<usize>,
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

        FretRange {
            range: beg_fret..end_fret,
        }
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
}

pub struct StringRange {
    range: Range<usize>,
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
            range: beg_string..end_string,
        }
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
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
    string_range: StringRange,
    fret_range: FretRange,
    notes: HashMap<(usize, usize), Note>,
}

impl ActiveNotes {
    fn new(
        registry: &NoteRegistry,
        tuning: &Tuning,
        string_range: StringRange,
        fret_range: FretRange,
    ) -> ActiveNotes {
        let mut notes = HashMap::new();
        for string_idx in string_range.range() {
            // TODO: read fret ranges while considering the tuning
            // TODO: read fret ranges while considering the tuning
            let open_note = tuning.note(string_idx);
            let mut note_iter = registry
                .iter_from(&open_note)
                .skip(fret_range.range().start);
            for fret_idx in fret_range.range() {
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

        ActiveNotes {
            string_range,
            fret_range,
            notes,
        }
    }
}
