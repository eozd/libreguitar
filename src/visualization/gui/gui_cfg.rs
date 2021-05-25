use serde::Deserialize;

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
