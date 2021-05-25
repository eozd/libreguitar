mod console_visualizer;
mod visualizer;
pub use console_visualizer::ConsoleVisualizer;
pub use visualizer::Visualizer;

#[cfg(feature = "gui")]
mod gui_visualizer;
#[cfg(feature = "gui")]
pub use gui_visualizer::{FrameData, GUIVisualizer};
