use crate::game_state::GameState;
use std::sync::mpsc;

pub struct ConsoleVisualizer {
    rx: mpsc::Receiver<GameState>,
}

impl ConsoleVisualizer {
    pub fn new(rx: mpsc::Receiver<GameState>) -> ConsoleVisualizer {
        ConsoleVisualizer { rx }
    }

    pub fn is_open(&self) -> bool {
        true
    }

    pub fn draw(&mut self) {
        let packet = self.rx.try_recv();
        if let Ok(note) = packet {
            println!("Play {:?}", note);
        }
    }
}
