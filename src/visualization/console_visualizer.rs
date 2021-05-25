use crate::core::{ConsoleCfg, FretLoc, FretRange, StringRange, Tuning};
use crate::game::GameState;
use crate::visualization::Visualizer;
use console::Term;
use std::error::Error;
use std::fmt;
use std::fmt::Write;
use std::sync::mpsc;

pub struct ConsoleVisualizer {
    rx: mpsc::Receiver<GameState>,
    fret_range: FretRange,
    string_range: StringRange,
    term: Term,
    previous_target: Option<FretLoc>,
    curr_target: FretLoc,
    fb_drawer: FretboardDrawer,
}

impl ConsoleVisualizer {
    pub fn new(
        rx: mpsc::Receiver<GameState>,
        fret_range: FretRange,
        string_range: StringRange,
        config: ConsoleCfg,
        tuning: Tuning,
    ) -> ConsoleVisualizer {
        let term = Term::stdout();
        let fb_drawer = FretboardDrawer {
            fret_size: config.fret_size,
            string_char: config.string_char,
            fret_char: config.fret_char,
            empty_char: config.empty_char,
            sep_str: config.sep_str,
            open_sep_str: config.open_sep_str,
            frets_to_number: config.frets_to_number,
            n_space_between_strings: config.n_space_between_strings,
            tuning,
        };
        ConsoleVisualizer {
            rx,
            fret_range,
            string_range,
            term,
            previous_target: None,
            curr_target: FretLoc {
                string_idx: 0,
                fret_idx: 0,
            },
            fb_drawer,
        }
    }
}

impl Visualizer for ConsoleVisualizer {
    fn is_open(&self) -> bool {
        true
    }

    fn draw(&mut self) {
        let packet = self.rx.try_recv();
        if let Ok(game_state) = packet {
            self.term.clear_screen().unwrap();
            self.term.write_line("Previously played note:").unwrap();
            if self.curr_target != game_state.target_loc {
                self.previous_target = Some(self.curr_target.clone());
                self.curr_target = game_state.target_loc.clone();
            }
            self.term
                .write_line(
                    &self
                        .fb_drawer
                        .draw(&self.fret_range, &self.string_range, &self.previous_target)
                        .unwrap(),
                )
                .unwrap();
            self.term
                .write_line(&format!(
                    "Play {} on string {} (detection count: {}/{})",
                    game_state.target_note.name_octave(),
                    game_state.target_loc.string_idx,
                    game_state.curr_detection_count,
                    game_state.needed_detection_count
                ))
                .unwrap();
        }
    }
}

struct FretboardDrawer {
    fret_size: usize,
    string_char: String,
    fret_char: String,
    empty_char: String,
    sep_str: String,
    open_sep_str: String,
    frets_to_number: Vec<usize>,
    n_space_between_strings: usize,
    tuning: Tuning,
}

impl FretboardDrawer {
    fn draw_fret(
        &self,
        out_str: &mut String,
        string_char: &str,
        fret_char: &str,
        is_fretted: bool,
    ) -> fmt::Result {
        debug_assert!(self.fret_size > 0, "Fret size must be positive");
        let left_side = self.fret_size / 2;
        let right_side = self.fret_size - left_side - (is_fretted as usize);
        write!(
            out_str,
            "{}",
            (0..left_side).map(|_| string_char).collect::<String>()
        )?;
        if is_fretted {
            write!(out_str, "{}", fret_char)?;
        }
        write!(
            out_str,
            "{}",
            (0..right_side).map(|_| string_char).collect::<String>()
        )?;
        Ok(())
    }

    fn draw_string(
        &self,
        out_str: &mut String,
        fret_range: &FretRange,
        played_fret: usize,
        open_note: &str,
    ) -> fmt::Result {
        let first_sep_char = if fret_range.r().start == 0 {
            &self.empty_char
        } else {
            &self.sep_str
        };
        write!(out_str, "{}", open_note)?;
        write!(out_str, "{}", first_sep_char)?;
        for i in fret_range.r() {
            self.draw_fret(
                out_str,
                &self.string_char,
                &self.fret_char,
                i == played_fret,
            )?;
            let sep_str = if i > 0 {
                &self.sep_str
            } else {
                &self.open_sep_str
            };
            write!(out_str, "{}", sep_str)?;
        }
        Ok(())
    }

    fn draw_fret_numbers(&self, out_str: &mut String, fret_range: &FretRange) -> fmt::Result {
        write!(out_str, "{}", self.empty_char)?;
        for i in fret_range.r() {
            let i_str = i.to_string();
            let i_in_first_octave = i % 12;
            self.draw_fret(
                out_str,
                &self.empty_char,
                &i_str,
                self.frets_to_number.contains(&i_in_first_octave),
            )?;
            write!(out_str, "{}", self.empty_char)?;
        }
        Ok(())
    }

    fn draw(
        &self,
        fret_range: &FretRange,
        string_range: &StringRange,
        target_loc: &Option<FretLoc>,
    ) -> Result<String, Box<dyn Error>> {
        let mut out = String::new();
        let out_of_bounds_fret = fret_range.r().end;
        let (fret_idx, string_idx) = match target_loc {
            Some(loc) => (loc.fret_idx, loc.string_idx),
            None => (out_of_bounds_fret, string_range.r().end),
        };
        for (i, open_note) in string_range.r().zip(self.tuning.iter()) {
            let fret_idx = if i == string_idx {
                fret_idx
            } else {
                out_of_bounds_fret
            };
            self.draw_string(&mut out, fret_range, fret_idx, &open_note.name.to_string())?;
            writeln!(&mut out)?;
            if i < string_range.r().end - 1 {
                for _ in 0..self.n_space_between_strings {
                    self.draw_string(&mut out, fret_range, out_of_bounds_fret, " ")?;
                    writeln!(&mut out)?;
                }
            }
        }
        write!(&mut out, " ")?;
        self.draw_fret_numbers(&mut out, &fret_range)?;
        Ok(out)
    }
}
