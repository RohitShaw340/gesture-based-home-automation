use std::{
    ops::{Deref, DerefMut},
    os::unix::net::UnixStream,
    sync::Arc,
};

use error_stack::{Result, ResultExt};
use flume::{unbounded, Receiver, Sender};
use serde::Deserialize;

use crate::{
    traits::{Responder, WantIpc},
    GError, HasGlamQuat, HasImagePosition, ImageFrame, ImageProcessor,
};

#[derive(Clone)]
pub struct HeadPoseEstimation {
    image_sender: Sender<(u32, u32, Arc<[u8]>)>,
    image_receiver: Receiver<(u32, u32, Arc<[u8]>)>,
    response_sender: Sender<error_stack::Result<HPEPreds, GError>>,
    response_receiver: Receiver<error_stack::Result<HPEPreds, GError>>,
    unix_stream: Arc<UnixStream>,
}

impl HeadPoseEstimation {
    pub fn new(unix_stream: UnixStream) -> Self {
        let (image_sender, image_receiver) = unbounded();
        let (response_sender, response_receiver) = unbounded();
        let unix_stream = Arc::new(unix_stream);

        Self {
            image_sender,
            image_receiver,
            response_sender,
            response_receiver,
            unix_stream,
        }
    }

    pub fn execute(&self, img: ImageFrame) -> error_stack::Result<HPEPreds, GError> {
        let ImageFrame {
            frame,
            width,
            height,
        } = img;
        self.send_ipc(&frame, width, height)?;
        let res = self.recv_ipc()?;
        serde_json::from_slice(&res).change_context(GError::IpcError)
    }

    pub fn send(&self, img: ImageFrame) -> Result<(), GError> {
        self.send_img(img.frame, img.width, img.height)
    }

    pub fn recv(&self) -> Result<HPEPreds, GError> {
        self.recv_response()?
    }
}

impl ImageProcessor for HeadPoseEstimation {
    fn image_sender(&self) -> &Sender<(u32, u32, Arc<[u8]>)> {
        &self.image_sender
    }

    fn image_receiver(&self) -> &Receiver<(u32, u32, Arc<[u8]>)> {
        &self.image_receiver
    }
}

impl Responder for HeadPoseEstimation {
    type Response = error_stack::Result<HPEPreds, GError>;

    fn response_sender(&self) -> &Sender<Self::Response> {
        &self.response_sender
    }

    fn response_receiver(&self) -> &Receiver<Self::Response> {
        &self.response_receiver
    }
}

impl WantIpc for HeadPoseEstimation {
    fn unix_stream(&self) -> &UnixStream {
        &self.unix_stream
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct HPEPreds {
    prediction: Vec<HpePrediction>,
}

impl Deref for HPEPreds {
    type Target = Vec<HpePrediction>;

    fn deref(&self) -> &Self::Target {
        &self.prediction
    }
}

impl DerefMut for HPEPreds {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.prediction
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct HpePrediction {
    pub x1: f32,
    pub x2: f32,
    pub y1: f32,
    pub y2: f32,
    pub conf: f32,
    pub class: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl HasImagePosition for HpePrediction {
    fn image_x(&self) -> f32 {
        (self.x1 + self.x2) / 2.0
    }

    fn image_y(&self) -> f32 {
        (self.y1 + self.y2) / 2.0
    }
}

impl HasGlamQuat for HpePrediction {
    fn quat(&self) -> glam::Quat {
        glam::Quat::from_euler(glam::EulerRot::ZYX, self.yaw, self.pitch, self.roll)
    }
}
