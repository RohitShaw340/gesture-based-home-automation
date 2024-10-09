use camera::CameraProc;
use config::Config;
use error_stack::Result;
use std::{
    collections::HashSet,
    fmt,
    io::Read,
    os::unix::net::{UnixListener, UnixStream},
    sync::{Arc, RwLock},
    thread,
};
use strum::EnumCount;

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

pub struct Models {
    pset: Arc<RwLock<HashSet<Process>>>,
    listener: UnixListener,
    hpe: Arc<RwLock<Option<HeadPoseEstimation>>>,
    gesture: Arc<RwLock<Option<GestureDetection>>>,
    head: Arc<RwLock<Option<HeadDetection>>>,
    cams: Arc<RwLock<Option<CameraProc>>>,
}

impl Models {
    pub fn new(listener: UnixListener) -> Self {
        Self {
            pset: Default::default(),
            hpe: Default::default(),
            gesture: Default::default(),
            head: Default::default(),
            cams: Default::default(),
            listener,
        }
    }

    pub fn hpe(&self) -> Result<HeadPoseEstimation, GError> {
        self.hpe
            .read()
            .unwrap()
            .clone()
            .ok_or(GError::ModelUninit.into())
    }

    pub fn gesture(&self) -> Result<GestureDetection, GError> {
        self.gesture
            .read()
            .unwrap()
            .clone()
            .ok_or(GError::ModelUninit.into())
    }

    pub fn head_detection(&self) -> Result<HeadDetection, GError> {
        self.head
            .read()
            .unwrap()
            .clone()
            .ok_or(GError::ModelUninit.into())
    }

    pub fn cams(&self) -> Result<CameraProc, GError> {
        self.cams
            .read()
            .unwrap()
            .clone()
            .ok_or(GError::ModelUninit.into())
    }

    pub fn add_process(&mut self, model: Process, stream: UnixStream, config: &Config) {
        match model {
            Process::HPE => {
                let model = HeadPoseEstimation::new(stream);
                let handle = model.run();
                {
                    *self.hpe.write().unwrap() = Some(model);
                }
                let hpe = self.hpe.clone();
                thread::spawn(move || {
                    // TODO: logging
                    handle.join();
                    *hpe.write().unwrap() = None;
                });
            }
            Process::GestureRecognition => {
                let model = GestureDetection::new(stream);
                let handle = model.run();
                {
                    *self.gesture.write().unwrap() = Some(model);
                }
                let gesture = self.gesture.clone();
                thread::spawn(move || {
                    // TODO: logging
                    handle.join();
                    *gesture.write().unwrap() = None;
                });
            }
            Process::HeadDetection => {
                let model = HeadDetection::new(stream);
                let handle = model.run();
                {
                    *self.head.write().unwrap() = Some(model);
                }
                let head = self.head.clone();
                thread::spawn(move || {
                    // TODO: logging
                    handle.join();
                    *head.write().unwrap() = None;
                });
            }
            Process::Camera => {
                let model = CameraProc::new(
                    stream,
                    config.camera1.img_width,
                    config.camera1.img_height,
                    config.camera2.img_width,
                    config.camera2.img_height,
                );
                let handle = model.run();
                {
                    *self.cams.write().unwrap() = Some(model);
                }
                let cams = self.cams.clone();
                thread::spawn(move || {
                    // TODO: logging
                    handle.join();
                    *cams.write().unwrap() = None;
                });
            }
        }

        self.pset.write().unwrap().insert(model);
    }

    pub fn len(&self) -> usize {
        self.pset.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn wait_for_connection(&mut self, config: &Config) {
        while self.len() < Process::COUNT {
            let (mut stream, _addr) = self.listener.accept().unwrap();

            let mut buffer = [0; 1024];
            let bytes_read = stream.read(&mut buffer).unwrap();
            let model: Process = String::from_utf8_lossy(&buffer[..bytes_read])
                .as_ref()
                .into();

            self.add_process(model, stream, config);
            println!("Processes connected: {}", self.len())
        }
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
}

impl App {
    pub fn new(config: Config, models: Models) -> App {
        App { config, models }
    }

    pub fn next(
        &self,
        frame1: ImageFrame,
        frame2: ImageFrame,
    ) -> error_stack::Result<Option<Vec<(config::Device, models::Gesture)>>, GError> {
        let (gestures, head_positions) = {
            // send frame1 to gesture detection model
            self.models.gesture()?.send(frame1.clone())?;
            // send frame2 to head detection model
            self.models.head_detection()?.send(frame2.clone())?;

            let mut head_positions = self.models.head_detection()?.recv()?;
            let mut gestures = self.models.gesture()?.recv()?;

            math::sort_horizontal(&mut head_positions);
            math::sort_horizontal(&mut gestures);

            (gestures, head_positions)
        };

        // proceed if any gesture is not none
        if gestures.iter().any(|x| !x.is_none()) {
            // send frame1 to hpe model
            self.models.hpe()?.send(frame1.clone())?;

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
                let mut hp = self.models.hpe().unwrap().recv().unwrap();
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
