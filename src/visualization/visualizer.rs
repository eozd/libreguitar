// DISCLAIMER: Major parts of the frame handling in this file is adapted
// from https://github.com/38/plotters/blob/master/examples/minifb-demo/src/main.rs
use crate::visualization::FrameData;
use minifb::{Key, Window, WindowOptions};
use plotters::chart::ChartState;
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;
use plotters_bitmap::bitmap_pixel::BGRXPixel;
use plotters_bitmap::BitMapBackend;
use std::borrow::{Borrow, BorrowMut};
use std::sync::mpsc;

const W: usize = 1280;
const H: usize = 960;

const FRAME_RATE: f64 = 30.0;
const FRAME_PERIOD: f64 = 1.0 / FRAME_RATE;

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

pub struct Visualizer {
    window: minifb::Window,
    buf: BufferWrapper,
    cs: ChartState<Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    xaxis: Vec<f64>,
    rx: mpsc::Receiver<FrameData>,
}

impl Visualizer {
    pub fn new(rx: mpsc::Receiver<FrameData>, xaxis_props: (f64, f64, f64)) -> Visualizer {
        let mut buf = BufferWrapper(vec![0u32; W * H]);

        let window = Window::new("Default Plotter Window", W, H, WindowOptions::default()).unwrap();
        let root = BitMapBackend::<BGRXPixel>::with_buffer_and_format(
            buf.borrow_mut(),
            (W as u32, H as u32),
        )
        .unwrap()
        .into_drawing_area();
        root.fill(&BLACK).unwrap();

        let (beg, end, step) = xaxis_props;
        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .set_all_label_area_size(30)
            .build_cartesian_2d(beg..2000.0, 0.0..0.01)
            .unwrap();

        chart
            .configure_mesh()
            .label_style(("sans-serif", 15).into_font().color(&GREEN))
            .axis_style(&GREEN)
            .draw()
            .unwrap();

        let cs = chart.into_chart_state();
        drop(root);
        Visualizer {
            window,
            buf,
            cs,
            xaxis: (beg..end).step(step).values().collect(),
            rx,
        }
    }

    pub fn animate(&mut self) {
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            let packet = self.rx.try_iter().last();
            if let Some(analysis) = packet {
                self.draw(&analysis.spectrogram);
            }
            std::thread::sleep(std::time::Duration::from_secs_f64(FRAME_PERIOD));
        }
    }

    fn draw(&mut self, arr: &[f64]) {
        let root = BitMapBackend::<BGRXPixel>::with_buffer_and_format(
            self.buf.borrow_mut(),
            (W as u32, H as u32),
        )
        .unwrap()
        .into_drawing_area();
        let mut chart = self.cs.clone().restore(&root);
        chart.plotting_area().fill(&BLACK).unwrap();

        chart
            .configure_mesh()
            .bold_line_style(&GREEN.mix(0.2))
            .light_line_style(&TRANSPARENT)
            .draw()
            .unwrap();

        let data = self.xaxis.iter().cloned().zip(arr.iter().cloned());
        chart.draw_series(LineSeries::new(data, &GREEN)).unwrap();

        drop(root);
        drop(chart);

        self.window.update_with_buffer(self.buf.borrow()).unwrap();
    }
}
