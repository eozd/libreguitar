use crate::game_logic::{FretRange, StringRange};
use crate::game_state::GameState;
use std::error::Error;
use std::fmt;
use std::fmt::Write;
use std::sync::mpsc;

pub struct ConsoleVisualizer {
    rx: mpsc::Receiver<GameState>,
    fret_range: FretRange,
    string_range: StringRange,
}

impl ConsoleVisualizer {
    pub fn new(
        rx: mpsc::Receiver<GameState>,
        fret_range: FretRange,
        string_range: StringRange,
    ) -> ConsoleVisualizer {
        ConsoleVisualizer {
            rx,
            fret_range,
            string_range,
        }
    }

    pub fn is_open(&self) -> bool {
        true
    }

    pub fn draw(&mut self) {
        let packet = self.rx.try_recv();
        if let Ok(note) = packet {
            println!("Play {:?}", note);
            let fb_repr = draw_fretboard(&self.fret_range, &self.string_range).unwrap();
            println!("{}", fb_repr);
        }
    }
}

fn draw_fret(
    out_str: &mut String,
    fret_size: usize,
    string_char: &str,
    fret_char: &str,
    is_fretted: bool,
) -> fmt::Result {
    debug_assert!(fret_size > 0, "Fret size must be positive");
    let left_side = fret_size / 2;
    let right_side = fret_size - left_side - (is_fretted as usize);
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
    out_str: &mut String,
    open_note: &str,
    fret_range: &FretRange,
    fret_size: usize,
    played_fret: usize,
    string_char: &str,
    sep_str: &str,
    open_sep_str: &str,
) -> fmt::Result {
    let first_sep_char = if fret_range.range().start == 0 {
        " "
    } else {
        sep_str
    };
    write!(out_str, "{}", open_note)?;
    write!(out_str, "{}", first_sep_char)?;
    for i in fret_range.range() {
        draw_fret(out_str, fret_size, string_char, "x", i == played_fret)?;
        let sep_str = if i > 0 { sep_str } else { open_sep_str };
        write!(out_str, "{}", sep_str)?;
    }
    Ok(())
}

fn draw_fret_numbers(
    out_str: &mut String,
    fret_range: &FretRange,
    frets_to_number: &[usize],
    fret_size: usize,
) -> fmt::Result {
    write!(out_str, " ")?;
    for i in fret_range.range() {
        let i_str = i.to_string();
        let i_in_first_octave = i % 12;
        draw_fret(
            out_str,
            fret_size,
            " ",
            &i_str,
            frets_to_number.contains(&i_in_first_octave),
        )?;
        write!(out_str, " ")?;
    }
    Ok(())
}

fn draw_fretboard(
    fret_range: &FretRange,
    string_range: &StringRange,
) -> Result<String, Box<dyn Error>> {
    let mut out = String::new();
    let string_char = "-";
    let empty_char = " ";
    let frets_to_number = vec![0, 3, 5, 7, 9];
    let fret_size = 5;
    let n_space_between_strings = 0;
    let sep_str = "|";
    let open_sep_str = "O";
    // TODO: get from Tuning struct
    let tuning = vec!["e", "b", "G", "D", "A", "E"];
    for (i, open_note) in string_range.range().zip(tuning.iter()) {
        draw_string(
            &mut out,
            open_note,
            fret_range,
            fret_size,
            50,
            string_char,
            sep_str,
            open_sep_str,
        )?;
        writeln!(&mut out)?;
        if i < string_range.range().end - 1 {
            for _ in 0..n_space_between_strings {
                draw_string(
                    &mut out,
                    " ",
                    fret_range,
                    fret_size,
                    50,
                    empty_char,
                    sep_str,
                    open_sep_str,
                )?;
                writeln!(&mut out)?;
            }
        }
    }
    write!(&mut out, " ")?;
    draw_fret_numbers(&mut out, &fret_range, &frets_to_number, fret_size)?;
    Ok(out)
}
