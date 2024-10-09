use std::os::unix::net::UnixListener;
use std::sync::Arc;
use std::time::Instant;

use gesture_ease::config::Config;
use gesture_ease::math::{
    angle_bw_cameras_from_z_axis, calc_position, get_closest_device_in_los_alt, get_los, sort_align,
};
use gesture_ease::models::{GesturePreds, HPEPreds, HeadPreds};
use gesture_ease::{App, GError, HasGlamQuat, HasImagePosition, Models};

use rppal::gpio::Gpio;

fn main() {
    let socket_path = "/tmp/gesurease.sock";
    let num_processes = 4;

    if std::fs::metadata(socket_path).is_ok() {
        // TODO: logging
        println!("Socket is already present. Deleting...");
        std::fs::remove_file(socket_path).unwrap();
    }

    let config = Config::open("config.toml".into()).unwrap();

    let listener = UnixListener::bind(socket_path).unwrap();
    let mut process_map = Models::new(num_processes, listener);

    let theta = angle_bw_cameras_from_z_axis(&config.camera1, &config.camera2);

    let gpio = Gpio::new().unwrap();

    for device in &config.devices {
        let mut pin = gpio.get(device.pin).unwrap().into_output();
        pin.set_reset_on_drop(false);
        pin.set_high();
    }

    process_map.wait_for_connection(&config);

    let app = App::new(config, process_map);

    let mut run = || -> error_stack::Result<(), GError> {
        let frames = process_map.cams()?.get()?;

        let frame1 = gesture_ease::ImageFrame {
            frame: frames.cam1.into(),
            width: config.camera1.img_width,
            height: config.camera1.img_height,
        };

        let frame2 = gesture_ease::ImageFrame {
            frame: frames.cam2.into(),
            width: config.camera2.img_width,
            height: config.camera2.img_height,
        };

        if let Some(devices) = app.next(frame1, frame2)? {
            for (device, gesture) in devices {
                // TODO: logging
                println!("gesture {:?} on device {}", gesture, device.name);
                let mut pin = gpio.get(device.pin).unwrap().into_output();
                pin.set_reset_on_drop(false);
                pin.toggle();
                // TODO: logging
                println!("pin state: {}", pin.is_set_low());
                //std::thread::sleep(std::time::Duration::from_secs(3));
            }
        }

        Ok(())
    };

    loop {
        let start = Instant::now();
        run().unwrap();
        let duration = Instant::now().duration_since(start).as_millis();
        // TODO: logging
        println!("duration in ms: {}", duration);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
