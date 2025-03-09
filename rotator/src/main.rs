use std::error::Error;

use clap::Parser;
use rotator::Servo;

// Period: 20 ms (50 Hz). Pulse width: min. 600 µs, neutral 1500 µs, max. 2250 µs.
const PERIOD_MS: u64 = 20;
const PULSE_MIN_US: u64 = 600;
const PULSE_NEUTRAL_US: u64 = 1500;
const PULSE_MAX_US: u64 = 2250;

const MIN_ANGLE: f64 = -80.0;
const MAX_ANGLE: f64 = 80.0;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    angle: f64,
    #[arg(short, long)]
    pin: u8,
    #[arg(long, default_value_t = MIN_ANGLE)]
    min_angle: f64,
    #[arg(long, default_value_t = MAX_ANGLE)]
    max_angle: f64,
    #[arg(long, default_value_t = PERIOD_MS)]
    period_ms: u64,
    #[arg(long, default_value_t = PULSE_MIN_US)]
    pulse_min_us: u64,
    #[arg(long, default_value_t = PULSE_NEUTRAL_US)]
    pulse_neutral_us: u64,
    #[arg(long, default_value_t = PULSE_MAX_US)]
    pulse_max_us: u64,
    #[arg(long, default_value_t = false)]
    hw_pwm: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let Args {
        angle,
        pin,
        min_angle,
        max_angle,
        period_ms,
        pulse_min_us,
        pulse_neutral_us,
        pulse_max_us,
        hw_pwm,
    } = Args::parse();

    let mut servo = Servo::new(
        pin,
        period_ms,
        pulse_min_us,
        pulse_neutral_us,
        pulse_max_us,
        min_angle,
        max_angle,
        hw_pwm,
    )?;

    servo.rotate(angle).inspect_err(|e| println!("[RotatorError] {e}"))?;

    Ok(())
}
