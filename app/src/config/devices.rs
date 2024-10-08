use std::sync::OnceLock;

use glam::Vec3A;
use rppal::gpio::{Gpio, OutputPin};
use rust_3d::{BoundingBox3D, HasBoundingBox3D, HasBoundingBox3DMaybe, Point3D};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

use crate::HasGlamPosition;

#[derive(Deserialize, Debug, Clone)]
pub struct Device {
    pub name: String,
    pub pin: u8,
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
    #[serde(skip)]
    pos: OnceLock<Vec3A>,
    #[serde(skip)]
    gpio: OnceLock<Arc<Mutex<OutputPin>>>,
}

impl Device {
    pub fn pos_mean(&self) -> &Vec3A {
        self.pos.get_or_init(|| {
            Vec3A::new(
                (self.min_x + self.max_x) / 2.0,
                (self.min_y + self.max_y) / 2.0,
                (self.min_z + self.max_z) / 2.0,
            )
        })
    }

    pub fn get_gpio(&self) -> Arc<Mutex<OutputPin>> {
        self.gpio
            .get_or_init(|| {
                let mut p = Gpio::new().unwrap().get(self.pin).unwrap().into_output();
                p.set_low();
                Arc::new(Mutex::new(p))
            })
            .clone()
    }
}

impl HasGlamPosition for Device {
    fn pos(&self) -> &Vec3A {
        self.pos.get_or_init(|| {
            Vec3A::new(
                (self.min_x + self.max_x) / 2.0,
                (self.min_y + self.max_y) / 2.0,
                (self.min_z + self.max_z) / 2.0,
            )
        })
    }
}

impl HasBoundingBox3DMaybe for Device {
    fn bounding_box_maybe(&self) -> rust_3d::Result<BoundingBox3D> {
        Ok(self.bounding_box())
    }
}

impl HasBoundingBox3D for Device {
    fn bounding_box(&self) -> BoundingBox3D {
        BoundingBox3D::new(
            &Point3D::new(self.min_x.into(), self.min_y.into(), self.min_z.into()),
            &Point3D::new(self.max_x.into(), self.max_y.into(), self.max_z.into()),
        )
        .unwrap()
    }
}
