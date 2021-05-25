pub trait Visualizer {
    fn draw(&mut self);
    fn is_open(&self) -> bool;
}
