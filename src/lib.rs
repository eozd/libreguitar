mod app;
mod audio_analysis;
mod core;
mod game;
mod visualization;

use crate::app::{App, AppError};
pub use crate::core::Cfg;

use cpal::Device;
use cpal::StreamConfig;

pub fn run(
    device: Device,
    device_config: StreamConfig,
    app_config: core::Cfg,
) -> Result<(), AppError> {
    let mut app = App::new(device, device_config, app_config)?;
    app.run()
}
