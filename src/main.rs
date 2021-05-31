#[macro_use]
extern crate log;
use simplelog::{CombinedLogger, ConfigBuilder as LogConfigBuilder, LevelFilter, WriteLogger};
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::iter;

use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::BufferSize;
use cpal::Device;
use cpal::Host;
use cpal::SampleRate;
use cpal::StreamConfig;

use libreguitar::{run, Cfg};

const APP_CONFIG_PATH: &str = "cfg";

fn choose_via_user_input<T>(title_str: &str, options: Vec<T>) -> io::Result<usize>
where
    T: Display,
{
    let n_choices = options.len();
    loop {
        println!("{}", title_str);
        println!(
            "{}",
            iter::repeat("-").take(title_str.len()).collect::<String>()
        );
        for (i, opt) in options.iter().enumerate() {
            println!("{}) {}", i, opt);
        }

        print!("Choose an option: ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        match buffer.trim().parse::<usize>() {
            Ok(choice) if choice < n_choices => return Ok(choice),
            _ => {
                println!("Invalid choice!");
            }
        }
    }
}

fn choose_host() -> Host {
    let hosts = cpal::available_hosts();
    let host_names = hosts.iter().map(|x| x.name()).collect();
    let host_id = choose_via_user_input("Available Hosts", host_names).unwrap();
    cpal::host_from_id(hosts[host_id]).unwrap()
}

fn choose_device(host: &Host) -> Device {
    let devices: Vec<Device> = host
        .input_devices()
        .expect("Could not get the list of devices")
        .collect();
    let device_names = devices
        .iter()
        .map(|x| match x.name() {
            Ok(name) => name,
            Err(_) => String::from("Unknown device"),
        })
        .collect();
    let device_id = choose_via_user_input("Available input devices", device_names).unwrap();
    devices
        .into_iter()
        .nth(device_id)
        .expect("Fatal error: User chose a device outside the range")
}

fn choose_device_config(_device: &Device) -> StreamConfig {
    // let supconfig = device.default_input_config().expect("No default config");
    // let config = supconfig.config();
    // TODO: choose from user
    StreamConfig {
        channels: 2,
        sample_rate: SampleRate(44100),
        buffer_size: BufferSize::Fixed(128),
    }
}

fn set_up_logger(log_path: &str) {
    let cfg = LogConfigBuilder::new().set_time_format_str("%FT%T").build();
    let out_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .unwrap();
    CombinedLogger::init(vec![WriteLogger::new(LevelFilter::Debug, cfg, out_file)]).unwrap();
}

fn main() {
    let app_config = Cfg::new(APP_CONFIG_PATH).unwrap();
    set_up_logger(&app_config.app.log_path);

    info!("Using app configs at {}", APP_CONFIG_PATH);

    let host = choose_host();
    info!("Using host {}", host.id().name());

    let device = choose_device(&host);
    info!("Using device {}", device.name().unwrap());

    let device_config = choose_device_config(&device);
    info!("Using device config {:?}", device_config);

    run(device, device_config, app_config).unwrap();
}
