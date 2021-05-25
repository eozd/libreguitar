mod console_visualizer;
mod visualizer;
pub use console_visualizer::ConsoleVisualizer;
pub use visualizer::Visualizer;

#[cfg(feature = "gui")]
mod gui;
#[cfg(feature = "gui")]
pub use gui::*;
