use camera::CameraProc;
use config::Config;
use error_stack::ResultExt;
use rppal::gpio::Gpio;
use std::{fmt, os::unix::net::UnixStream, sync::Arc};
use threadpool::ThreadPool;
use traits::Responder;

use models::{GestureDetection, HeadDetection, HeadPoseEstimation};

mod error;

pub mod camera;
pub mod config;
pub mod math;
pub mod models;
pub mod traits;

pub use error::GError;
pub use traits::{HasGlamPosition, HasGlamQuat, HasImagePosition, ImageProcessor};

pub struct ImageCoords {
    pub x: f32,
    pub y: f32,
    w: f32,
    h: f32,
}

impl ImageCoords {
    pub fn new(x: f32, y: f32, w: u32, h: u32) -> Self {
        Self {
            x,
            y,
            w: w as f32,
            h: h as f32,
        }
    }

    pub fn x_max(&self) -> f32 {
        self.w
    }

    pub fn y_max(&self) -> f32 {
        self.h
    }

    pub fn x_mid(&self) -> f32 {
        self.x_max() / 2.0
    }

    pub fn y_mid(&self) -> f32 {
        self.y_max() / 2.0
    }

    pub fn coords_from_mid(&self) -> (f32, f32) {
        (self.x - self.x_mid(), self.y - self.y_mid())
    }
}

#[derive(strum_macros::EnumCount, PartialEq, Eq, Hash)]
pub enum Process {
    HPE,
    GestureRecognition,
    HeadDetection,
    Camera,
}

impl Process {
    // TODO: instead of harcoding, add this to some config
    pub fn addr(&self) -> &str {
        match self {
            Self::Camera => "/tmp/picam.sock",
            Self::HPE => "/tmp/hpe.sock",
            Self::HeadDetection => "/tmp/head.sock",
            Self::GestureRecognition => "/tmp/gesture.sock",
        }
    }

    pub fn connect(&self) -> error_stack::Result<UnixStream, GError> {
        UnixStream::connect(self.addr()).change_context(GError::ConnectionError)
    }

    pub fn connect_at(addr: impl AsRef<str>) -> error_stack::Result<UnixStream, GError> {
        UnixStream::connect(addr.as_ref()).change_context(GError::ConnectionError)
    }
}

impl From<&str> for Process {
    fn from(value: &str) -> Self {
        match value {
            "hpe" | "directmhp" => Self::HPE,
            "ge" | "gesture" => Self::GestureRecognition,
            "head" => Self::HeadDetection,
            "cam" => Self::Camera,
            _ => panic!("invalid"),
        }
    }
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HPE => write!(f, "hpe"),
            Self::HeadDetection => write!(f, "head"),
            Self::GestureRecognition => write!(f, "gesture"),
            Self::Camera => write!(f, "cam"),
        }
    }
}

#[derive(Clone)]
pub struct Models {
    pub hpe: HeadPoseEstimation,
    pub gesture: GestureDetection,
    pub head: HeadDetection,
    pub cams: CameraProc,
}

impl Models {
    pub fn new() -> error_stack::Result<Self, GError> {
        let hpe = HeadPoseEstimation::new(Process::HPE.connect()?);
        let gesture = GestureDetection::new(Process::GestureRecognition.connect()?);
        let head = HeadDetection::new(Process::HeadDetection.connect()?);
        let cams = CameraProc::new(Process::Camera.connect()?);
        Ok(Self {
            hpe,
            gesture,
            head,
            cams,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ImageFrame {
    pub frame: Arc<[u8]>,
    pub width: u32,
    pub height: u32,
}

pub struct App {
    pub config: Config,
    pub models: Models,
    pub pool: ThreadPool,
}

impl App {
    pub fn new(config: Config) -> error_stack::Result<App, GError> {
        Ok(App {
            config,
            models: Models::new()?,
            pool: ThreadPool::new(4),
        })
    }

    pub fn hpe(&self, frame: ImageFrame) {
        let models = self.models.clone();
        self.pool.execute(move || {
            let res = models.hpe.execute(frame);
            models
                .hpe
                .send_response(res)
                .change_context(GError::CommError)
                .unwrap();
        });
    }

    pub fn gesture(&self, frame: ImageFrame) {
        let models = self.models.clone();
        self.pool.execute(move || {
            let res = models.gesture.execute(frame);
            models
                .gesture
                .send_response(res)
                .change_context(GError::CommError)
                .unwrap();
        });
    }

    pub fn head(&self, frame: ImageFrame) {
        let models = self.models.clone();
        self.pool.execute(move || {
            let res = models.head.execute(frame);
            models
                .head
                .send_response(res)
                .change_context(GError::CommError)
                .unwrap();
        });
    }

    pub fn run(&self) -> error_stack::Result<(), GError> {
        // TODO: move this to device
        let gpio = Gpio::new().change_context(GError::GpioError)?;

        let camera::Frames {
            cam1: frame1,
            cam2: frame2,
        } = self.models.cams.get_frames()?;

        if let Some(devices) = self.next(frame1, frame2)? {
            for (device, gesture) in devices {
                // TODO: logging
                println!("gesture {:?} on device {}", gesture, device.name);
                let mut pin = gpio
                    .get(device.pin)
                    .change_context(GError::GpioError)?
                    .into_output();
                pin.set_reset_on_drop(false);
                pin.toggle();
                // TODO: logging
                println!("pin state: {}", pin.is_set_low());
            }
        }

        Ok(())
    }

    pub fn next(
        &self,
        frame1: ImageFrame,
        frame2: ImageFrame,
    ) -> error_stack::Result<Option<Vec<(config::Device, models::Gesture)>>, GError> {
        let (gestures, head_positions) = {
            // send frame1 to gesture detection model
            self.gesture(frame1.clone());
            // send frame2 to head detection model
            self.head(frame2.clone());

            let mut head_positions = self.models.head.recv()?;
            let mut gestures = self.models.gesture.recv()?;

            math::sort_horizontal(&mut head_positions);
            math::sort_horizontal(&mut gestures);

            (gestures, head_positions)
        };

        // proceed if any gesture is not none
        if gestures.iter().any(|x| !x.is_none()) {
            // send frame1 to hpe model
            self.hpe(frame1.clone());

            // in the meantime calculate positition of head which had a gesture
            let positions = gestures.iter().zip(head_positions.iter()).map(|(g, h)| {
                if !g.is_none() {
                    Some((
                        math::calc_position(
                            &self.config.camera1,
                            &g.image_coords(frame1.width, frame1.height),
                            &self.config.camera2,
                            &h.image_coords(frame2.width, frame2.height),
                        )
                        .unwrap(),
                        g.gesture.clone(),
                    ))
                } else {
                    None
                }
            });

            let head_poses = {
                let mut hp = self.models.hpe.recv()?;
                math::sort_horizontal(&mut hp);
                hp
            };

            // Now get the device in line of sight of each head
            return Ok(Some(
                head_poses
                    .iter()
                    .zip(positions)
                    .filter_map(|(pose, position)| {
                        let (position, gesture) = if let Some((position, gesture)) = position {
                            (position, gesture)
                        } else {
                            return None;
                        };

                        let line_of_sight =
                            math::get_los(&self.config.camera1, &position, &pose.quat());
                        math::get_closest_device_in_los_alt(&self.config, line_of_sight)
                            .map(|x| (x, gesture))
                    })
                    .collect(),
            ));
        }

        Ok(None)
    }
}
