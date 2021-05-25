// DISCLAIMER: Major parts of the frame handling in this file is adapted
// from https://github.com/38/plotters/blob/master/examples/minifb-demo/src/main.rs
use crate::visualization::gui::GuiCfg;
use crate::visualization::Visualizer;
use minifb::{Key, Window, WindowOptions};
use plotters::chart::ChartState;
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;
use plotters::style::RGBAColor;
use plotters_bitmap::bitmap_pixel::BGRXPixel;
use plotters_bitmap::BitMapBackend;
use std::borrow::{Borrow, BorrowMut};
use std::sync::mpsc;

struct BufferWrapper(Vec<u32>);
impl Borrow<[u8]> for BufferWrapper {
    fn borrow(&self) -> &[u8] {
        // Safe for alignment: align_of(u8) <= align_of(u32)
        // Safe for cast: u32 can be thought of as being transparent over [u8; 4]
        unsafe { std::slice::from_raw_parts(self.0.as_ptr() as *const u8, self.0.len() * 4) }
    }
}
impl BorrowMut<[u8]> for BufferWrapper {
    fn borrow_mut(&mut self) -> &mut [u8] {
        // Safe for alignment: align_of(u8) <= align_of(u32)
        // Safe for cast: u32 can be thought of as being transparent over [u8; 4]
        unsafe { std::slice::from_raw_parts_mut(self.0.as_mut_ptr() as *mut u8, self.0.len() * 4) }
    }
}
impl Borrow<[u32]> for BufferWrapper {
    fn borrow(&self) -> &[u32] {
        self.0.as_slice()
    }
}
impl BorrowMut<[u32]> for BufferWrapper {
    fn borrow_mut(&mut self) -> &mut [u32] {
        self.0.as_mut_slice()
    }
}

fn color_from_tup(rgb: (u8, u8, u8, u8)) -> RGBAColor {
    let alpha = rgb.3 as f64 / 255.0;
    RGBColor(rgb.0, rgb.1, rgb.2).mix(alpha)
}

pub struct FrameData {
    pub spectrogram: Vec<f64>,
}

pub struct GUIVisualizer {
    window: minifb::Window,
    buf: BufferWrapper,
    cs: ChartState<Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    xaxis: Vec<f64>,
    rx: mpsc::Receiver<FrameData>,
    gui_cfg: GuiCfg,
    background_color: RGBAColor,
    line_color: RGBAColor,
}

impl GUIVisualizer {
    pub fn new(
        rx: mpsc::Receiver<FrameData>,
        xaxis_props: (f64, f64, f64),
        gui_cfg: GuiCfg,
    ) -> GUIVisualizer {
        let w = gui_cfg.width;
        let h = gui_cfg.height;
        let font_color = color_from_tup(gui_cfg.font_color);
        let axis_color = color_from_tup(gui_cfg.axis_color);
        let background_color = color_from_tup(gui_cfg.background_color);
        let line_color = color_from_tup(gui_cfg.line_color);
        let mut buf = BufferWrapper(vec![0u32; w * h]);

        let window = Window::new("Default Plotter Window", w, h, WindowOptions::default()).unwrap();
        let root = BitMapBackend::<BGRXPixel>::with_buffer_and_format(
            buf.borrow_mut(),
            (w as u32, h as u32),
        )
        .unwrap()
        .into_drawing_area();
        root.fill(&background_color).unwrap();

        let (beg, end, step) = xaxis_props;
        let mut chart = ChartBuilder::on(&root)
            .margin(gui_cfg.margin_size)
            .set_all_label_area_size(gui_cfg.label_area_size)
            .build_cartesian_2d(
                beg..gui_cfg.spectrum_max_freq,
                0.0..gui_cfg.spectrum_max_magnitude,
            )
            .unwrap();

        let fonttup = (&gui_cfg.font_name[..], gui_cfg.font_size);
        chart
            .configure_mesh()
            .label_style(fonttup.into_font().color(&font_color))
            .axis_style(&axis_color)
            .draw()
            .unwrap();

        let cs = chart.into_chart_state();
        drop(root);
        GUIVisualizer {
            window,
            buf,
            cs,
            xaxis: (beg..end).step(step).values().collect(),
            rx,
            gui_cfg,
            background_color,
            line_color,
        }
    }
}

impl Visualizer for GUIVisualizer {
    fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    fn draw(&mut self) {
        let packet = self.rx.try_iter().last();
        if packet.is_none() {
            return;
        }
        let arr = packet.unwrap().spectrogram;
        let root = BitMapBackend::<BGRXPixel>::with_buffer_and_format(
            self.buf.borrow_mut(),
            (self.gui_cfg.width as u32, self.gui_cfg.height as u32),
        )
        .unwrap()
        .into_drawing_area();
        let mut chart = self.cs.clone().restore(&root);
        chart.plotting_area().fill(&self.background_color).unwrap();

        chart
            .configure_mesh()
            .bold_line_style(&self.line_color)
            .light_line_style(&TRANSPARENT)
            .draw()
            .unwrap();

        let data = self.xaxis.iter().cloned().zip(arr.iter().cloned());
        chart
            .draw_series(LineSeries::new(data, &self.line_color))
            .unwrap();

        drop(root);
        drop(chart);

        self.window.update_with_buffer(self.buf.borrow()).unwrap();
    }
}
