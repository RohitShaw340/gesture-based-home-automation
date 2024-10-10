use std::{
    ops::{Deref, DerefMut},
    os::unix::net::UnixStream,
    sync::Arc,
    thread::{self, JoinHandle},
};

use error_stack::{Result, ResultExt};
use flume::{unbounded, Receiver, Sender};
use serde::Deserialize;

use crate::{
    traits::{Responder, WantIpc},
    GError, HasImagePosition, ImageFrame, ImageProcessor,
};

#[derive(Clone)]
pub struct HeadDetection {
    image_sender: Sender<(u32, u32, Arc<[u8]>)>,
    image_receiver: Receiver<(u32, u32, Arc<[u8]>)>,
    response_sender: Sender<error_stack::Result<HeadPreds, GError>>,
    response_receiver: Receiver<error_stack::Result<HeadPreds, GError>>,
    unix_stream: Arc<UnixStream>,
}

impl HeadDetection {
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

    pub fn execute(&self, img: ImageFrame) -> error_stack::Result<HeadPreds, GError> {
        let ImageFrame {
            frame,
            width,
            height,
        } = img;

        self.send_ipc(&frame, width, height)?;
        let res = self.recv_ipc()?;
        serde_json::from_slice(&res).change_context(GError::IpcError)
    }

    pub fn run(&self) -> JoinHandle<error_stack::Result<(), GError>> {
        let instance = self.clone();
        // TODO: logging
        println!("Head Detection model connected");

        thread::spawn(move || loop {
            let (w, h, _img) = instance.recv_img()?;

            instance.send_ipc(&_img, w, h)?;
            let res = instance.recv_ipc()?;
            let res: HeadPreds = serde_json::from_slice(&res).change_context(GError::IpcError)?;

            //instance.send_response(res)?;
        })
    }

    pub fn send(&self, img: ImageFrame) -> Result<(), GError> {
        self.send_img(img.frame, img.width, img.height)
    }

    pub fn recv(&self) -> Result<HeadPreds, GError> {
        self.recv_response()?
    }
}

impl ImageProcessor for HeadDetection {
    fn image_sender(&self) -> &Sender<(u32, u32, Arc<[u8]>)> {
        &self.image_sender
    }

    fn image_receiver(&self) -> &Receiver<(u32, u32, Arc<[u8]>)> {
        &self.image_receiver
    }
}

impl Responder for HeadDetection {
    type Response = error_stack::Result<HeadPreds, GError>;

    fn response_sender(&self) -> &Sender<Self::Response> {
        &self.response_sender
    }

    fn response_receiver(&self) -> &Receiver<Self::Response> {
        &self.response_receiver
    }
}

impl WantIpc for HeadDetection {
    fn unix_stream(&self) -> &UnixStream {
        &self.unix_stream
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct HeadPreds {
    pub prediction: Vec<HeadPrediction>,
}

impl Deref for HeadPreds {
    type Target = Vec<HeadPrediction>;

    fn deref(&self) -> &Self::Target {
        &self.prediction
    }
}

impl DerefMut for HeadPreds {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.prediction
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct HeadPrediction {
    pub nose_x: f32,
    pub nose_y: f32,
}

impl HasImagePosition for HeadPrediction {
    fn image_y(&self) -> f32 {
        self.nose_y
    }

    fn image_x(&self) -> f32 {
        self.nose_x
    }
}
