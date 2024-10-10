use crate::traits::{GenProcess, Responder, WantIpc};
use crate::{GError, ImageFrame};
use error_stack::Result;
use flume::unbounded;
use flume::{Receiver, Sender};
use std::{os::unix::net::UnixStream, sync::Arc};

#[derive(Clone)]
pub struct CameraProc {
    data_sender: Sender<u32>,
    data_receiver: Receiver<u32>,
    response_sender: Sender<Frames>,
    response_receiver: Receiver<Frames>,
    unix_stream: Arc<UnixStream>,
}

impl CameraProc {
    pub fn new(unix_stream: UnixStream) -> Self {
        let (data_sender, data_receiver) = unbounded();
        let (response_sender, response_receiver) = unbounded();
        let unix_stream = Arc::new(unix_stream);

        Self {
            data_sender,
            data_receiver,
            response_sender,
            response_receiver,
            unix_stream,
        }
    }

    pub fn get_frames(&self) -> error_stack::Result<Frames, GError> {
        self.send_u32(1)?;
        let w1 = self.recv_u32()?;
        let h1 = self.recv_u32()?;
        let img1 = self.recv_ipc()?;
        self.send_u32(2)?;
        let w2 = self.recv_u32()?;
        let h2 = self.recv_u32()?;
        let img2 = self.recv_ipc()?;

        let img1 = ImageFrame {
            frame: img1.into(),
            width: w1,
            height: h1,
        };

        let img2 = ImageFrame {
            frame: img2.into(),
            width: w2,
            height: h2,
        };

        Ok(Frames {
            cam1: img1,
            cam2: img2,
        })
    }

    pub fn get(&self) -> Result<Frames, GError> {
        self.send_data(1)?;
        self.recv_response()
    }
}

impl GenProcess for CameraProc {
    type Send = u32;

    fn data_sender(&self) -> &Sender<Self::Send> {
        &self.data_sender
    }

    fn data_receiver(&self) -> &Receiver<Self::Send> {
        &self.data_receiver
    }
}

impl Responder for CameraProc {
    type Response = Frames;

    fn response_sender(&self) -> &Sender<Self::Response> {
        &self.response_sender
    }

    fn response_receiver(&self) -> &Receiver<Self::Response> {
        &self.response_receiver
    }
}

impl WantIpc for CameraProc {
    fn unix_stream(&self) -> &UnixStream {
        &self.unix_stream
    }
}

#[derive(Debug)]
pub struct Frames {
    pub cam1: ImageFrame,
    pub cam2: ImageFrame,
}
