#[cfg(feature = "gui")]
use crate::visualization::GuiCfg;
use config::{Config, ConfigError, File};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppCfg {
    pub fps: f64,
    pub frequencies_path: String,
    pub tuning_path: String,
    pub block_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct ConsoleCfg {
    pub fret_size: usize,
    pub string_char: String,
    pub fret_char: String,
    pub empty_char: String,
    pub sep_str: String,
    pub open_sep_str: String,
    pub frets_to_number: Vec<usize>,
    pub n_space_between_strings: usize,
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
    pub state_update_period: usize,
}

#[derive(Debug, Deserialize)]
pub struct Cfg {
    pub app: AppCfg,
    pub audio: AudioCfg,
    pub game: GameCfg,
    pub console: ConsoleCfg,
    #[cfg(feature = "gui")]
    pub gui: GuiCfg,
}

fn get_cfg<T>(path: &str) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let mut s = Config::default();
    s.merge(File::with_name(path))?;
    s.try_into()
}

impl Cfg {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        let base_path = Path::new(path);
        let app_cfg = get_cfg(base_path.join(Path::new("app.toml")).to_str().unwrap())?;
        let audio_cfg = get_cfg(base_path.join(Path::new("audio.toml")).to_str().unwrap())?;
        let game_cfg = get_cfg(base_path.join(Path::new("game.toml")).to_str().unwrap())?;
        let console_cfg = get_cfg(base_path.join(Path::new("console.toml")).to_str().unwrap())?;

        Ok(Cfg {
            app: app_cfg,
            audio: audio_cfg,
            game: game_cfg,
            console: console_cfg,
            #[cfg(feature = "gui")]
            gui: get_cfg(base_path.join(Path::new("gui.toml")).to_str().unwrap())?,
        })
    }
}
