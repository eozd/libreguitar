use std::io::{self, Write};

use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::traits::StreamTrait;

fn main() {
    let host_ids = cpal::available_hosts();
    println!("Available hosts");
    println!("---------------");
    for host_id in host_ids.iter() {
        println!("ID: {}", host_id.name());
    }

    let host = cpal::default_host();
    println!("Using host {}", host.id().name());
    let devices: Vec<cpal::Device> = host
        .input_devices()
        .expect("Could not get the list of devices")
        .collect();

    println!();
    println!("Available input devices");
    println!("-----------------------");
    for (i, device) in devices.iter().enumerate() {
        let name = match device.name() {
            Ok(name) => name,
            Err(_) => String::from("Error: Device has no name"),
        };
        println!("Device {}: {}", i, name);
    }

    let default_device = host
        .default_input_device()
        .expect("No input device available");
    println!(
        "Default device: {}",
        default_device.name().expect("Can't get device name")
    );

    let mut buffer = String::new();
    print!("Choose a device (Enter to skip): ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut buffer)
        .expect("Could not read from STDIN");
    let device;
    let buffer = buffer.trim();
    if buffer.len() == 0 {
        device = &default_device;
    } else {
        let chosen_device_id: usize = buffer.parse().unwrap();
        assert!(chosen_device_id < devices.len(), "Invalid device ID");
        device = &devices[chosen_device_id];
    }

    let supconfig = device.default_input_config().expect("No default config");
    let config = supconfig.config();
    println!("Config {:?}", supconfig);
    // TODO: When using Buffersize::Fixed, alsa set_hw_params crashes. Find a fix.
    // let config = StreamConfig {
    //     channels: 2,
    //     sample_rate: SampleRate(44100),
    //     buffer_size: BufferSize::Fixed(128),
    // };
    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                println!("Read {} elements", data.len());
            },
            move |_err| {
                println!("Error reading data from device");
            },
        )
        .expect("Could not build stream");
    println!("Playing device...");
    stream.play().expect("Error playing the device...");
    std::thread::sleep(std::time::Duration::from_secs(10));
}
