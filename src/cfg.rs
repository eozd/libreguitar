use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppCfg {
    pub fps: f64,
    pub frequencies_path: String,
    pub tuning_path: String,
    pub block_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct GuiCfg {
    pub width: usize,
    pub height: usize,
    pub margin_size: u32,
    pub label_area_size: u32,
    pub spectrum_max_freq: f64,
    pub spectrum_max_magnitude: f64,
    pub font_name: String,
    pub font_size: i32,
    pub font_color: (u8, u8, u8, u8),
    pub axis_color: (u8, u8, u8, u8),
    pub background_color: (u8, u8, u8, u8),
    pub line_color: (u8, u8, u8, u8),
}

#[derive(Debug, Deserialize)]
pub struct AudioCfg {
    pub fft_res_factor: f64,
    pub fft_magnitude_gain: f64,
    pub peak_threshold: f64,
    pub min_peak_dist: usize,
    pub num_top_peaks: usize,
    pub moving_avg_window_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct GameCfg {
    pub fret_range: (usize, usize),
    pub string_range: (usize, usize),
    pub note_count_for_acceptance: usize,
}

#[derive(Debug, Deserialize)]
pub struct Cfg {
    pub app: AppCfg,
    pub gui: GuiCfg,
    pub audio: AudioCfg,
    pub game: GameCfg,
}

impl Cfg {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        let mut s = Config::default();

        s.merge(File::with_name(path))?;
        s.try_into()
    }
}
