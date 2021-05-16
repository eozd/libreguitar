use crate::visualization::ConsoleData;
use std::sync::mpsc;

pub struct ConsoleVisualizer {
    rx: mpsc::Receiver<ConsoleData>,
}

impl ConsoleVisualizer {
    pub fn new(rx: mpsc::Receiver<ConsoleData>) -> ConsoleVisualizer {
        ConsoleVisualizer { rx }
    }

    pub fn is_open(&self) -> bool {
        true
    }

    pub fn draw(&mut self) {
        let packet = self.rx.try_iter().last();
        if let None = packet {
            return;
        }
        let maybe_note = packet.unwrap().note;
        if let Some(note) = maybe_note {
            println!("Detected note: {:?}", note);
        }
    }
}
