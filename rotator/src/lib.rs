use std::time::Duration;

use rppal::gpio::Gpio;

pub struct Servo {
    pin: rppal::gpio::OutputPin,
    pub period_ms: u64,
    pub pulse_min_us: u64,
    pub pulse_neutral_us: u64,
    pub pulse_max_us: u64,
    pub hw_pwm: bool,
    pub min_angle: f64,
    pub max_angle: f64,
}

impl Servo {
    pub fn new(
        gpio_pwm: u8,
        period_ms: u64,
        pulse_min_us: u64,
        pulse_neutral_us: u64,
        pulse_max_us: u64,
        min_angle: f64,
        max_angle: f64,
        hw_pwm: bool,
    ) -> Result<Self, rppal::gpio::Error> {
        let gpio = Gpio::new()?;
        let mut pin = gpio.get(gpio_pwm)?.into_output();
        Ok(Self {
            pin,
            period_ms,
            pulse_min_us,
            pulse_neutral_us,
            pulse_max_us,
            hw_pwm,
            min_angle,
            max_angle,
        })
    }

    pub fn rotate(&mut self, angle: f64) -> Result<(), rppal::gpio::Error> {
        if self.hw_pwm {
            self.rotate_hwpwm(angle)
        } else {
            self.rotate_sfpwm(angle)
        }
    }

    fn rotate_sfpwm(&mut self, angle: f64) -> Result<(), rppal::gpio::Error> {
        self.pin.set_pwm(
            Duration::from_millis(self.period_ms),
            Duration::from_micros(self.angle_to_pulse_width(angle)),
        )?;

        Ok(())
    }

    fn rotate_hwpwm(&mut self, angle: f64) -> Result<(), rppal::gpio::Error> {
        todo!()
    }

    fn angle_to_pulse_width(&self, angle: f64) -> u64 {
        // TODO: handle error
        assert!(self.min_angle >= angle);
        assert!(self.max_angle <= angle);
        let mid = (self.max_angle - self.min_angle) / 2.0;
        (if angle < mid {
            self.pulse_min_us as f64
                + (angle - self.min_angle) * (self.pulse_neutral_us - self.pulse_min_us) as f64
        } else {
            self.pulse_max_us as f64
                - (self.max_angle  - angle) * (self.pulse_max_us - self.pulse_neutral_us) as f64
        }) as u64
    }
}
