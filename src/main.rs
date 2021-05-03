use std::fmt::Display;
use std::io::{self, Write};
use std::iter;

use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::traits::StreamTrait;
use cpal::BufferSize;
use cpal::Device;
use cpal::Host;
use cpal::SampleRate;
use cpal::Stream;
use cpal::StreamConfig;

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

fn choose_config(device: &Device) -> StreamConfig {
    // let supconfig = device.default_input_config().expect("No default config");
    // let config = supconfig.config();
    // TODO: choose from user
    StreamConfig {
        channels: 2,
        sample_rate: SampleRate(44100),
        buffer_size: BufferSize::Fixed(128),
    }
}

fn build_stream(device: &Device, config: &StreamConfig) -> Stream {
    device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                println!(
                    "Maximum is {:?}",
                    data.iter().cloned().fold(0. / 0., f32::max)
                );
            },
            move |_err| {
                println!("Error reading data from device");
            },
        )
        .expect("Could not build stream")
}

fn main() {
    let host = choose_host();
    println!("Using host {}", host.id().name());

    let device = choose_device(&host);
    println!("Using device {}", device.name().unwrap());

    let config = choose_config(&device);
    println!("Using config {:?}", config);

    let stream = build_stream(&device, &config);
    println!("Playing device...");
    stream.play().expect("Error playing the device...");
    std::thread::sleep(std::time::Duration::from_secs(1000));
}
