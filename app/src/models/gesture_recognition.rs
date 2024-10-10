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
    GError, HasImagePosition, ImageFrame, ImageProcessor,
};

#[derive(Clone)]
pub struct GestureDetection {
    image_sender: Sender<(u32, u32, Arc<[u8]>)>,
    image_receiver: Receiver<(u32, u32, Arc<[u8]>)>,
    response_sender: Sender<error_stack::Result<GesturePreds, GError>>,
    response_receiver: Receiver<error_stack::Result<GesturePreds, GError>>,
    unix_stream: Arc<UnixStream>,
}

impl GestureDetection {
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

    pub fn execute(&self, img: ImageFrame) -> error_stack::Result<GesturePreds, GError> {
        let ImageFrame {
            frame,
            width,
            height,
        } = img;

        self.send_ipc(&frame, width, height)?;
        let res = self.recv_ipc()?;
        serde_json::from_slice(&res).change_context(GError::IpcError)
    }

    pub fn send(&self, img: crate::ImageFrame) -> Result<(), GError> {
        self.send_img(img.frame, img.width, img.height)
    }

    pub fn recv(&self) -> Result<GesturePreds, GError> {
        self.recv_response()?
    }
}

impl ImageProcessor for GestureDetection {
    fn image_sender(&self) -> &Sender<(u32, u32, Arc<[u8]>)> {
        &self.image_sender
    }

    fn image_receiver(&self) -> &Receiver<(u32, u32, Arc<[u8]>)> {
        &self.image_receiver
    }
}

impl Responder for GestureDetection {
    type Response = error_stack::Result<GesturePreds, GError>;

    fn response_sender(&self) -> &Sender<Self::Response> {
        &self.response_sender
    }

    fn response_receiver(&self) -> &Receiver<Self::Response> {
        &self.response_receiver
    }
}

impl WantIpc for GestureDetection {
    fn unix_stream(&self) -> &UnixStream {
        &self.unix_stream
    }
}

#[derive(Default, Debug, Deserialize, Clone)]
pub struct GesturePreds {
    pub prediction: Vec<GesturePrediction>,
}

impl Deref for GesturePreds {
    type Target = Vec<GesturePrediction>;

    fn deref(&self) -> &Self::Target {
        &self.prediction
    }
}

impl DerefMut for GesturePreds {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.prediction
    }
}

#[derive(Default, Debug, Deserialize, Clone, PartialEq, Eq, strum_macros::EnumIs)]
pub enum Gesture {
    Toggle,
    #[default]
    None,
}

#[derive(Default, Debug, Deserialize, Clone, PartialEq)]
pub struct GesturePrediction {
    pub nose_x: f32,
    pub nose_y: f32,
    pub gesture: Gesture,
}

impl Deref for GesturePrediction {
    type Target = Gesture;

    fn deref(&self) -> &Self::Target {
        &self.gesture
    }
}

impl HasImagePosition for GesturePrediction {
    fn image_x(&self) -> f32 {
        self.nose_x
    }

    fn image_y(&self) -> f32 {
        self.nose_y
    }
}
