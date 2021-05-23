mod app;
mod audio_analysis;
mod cfg;
mod game_logic;
mod game_state;
mod note;
mod visualization;

use crate::app::{App, AppError};

use cpal::Device;
use cpal::StreamConfig;

pub use crate::cfg::{AppCfg, AudioCfg, Cfg, GameCfg, GuiCfg};

pub fn run(device: Device, device_config: StreamConfig, app_config: Cfg) -> Result<(), AppError> {
    let mut app = App::new(device, device_config, app_config)?;
    app.run()
}
