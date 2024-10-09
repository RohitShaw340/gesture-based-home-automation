use crate::traits::{GenProcess, Responder, WantIpc};
use crate::GError;
use error_stack::Result;
use flume::unbounded;
use flume::{Receiver, Sender};
use serde::Deserialize;
use std::{
    os::unix::net::UnixStream,
    sync::Arc,
    thread::{self, JoinHandle},
};

#[derive(Clone)]
pub struct CameraProc {
    data_sender: Sender<u32>,
    data_receiver: Receiver<u32>,
    response_sender: Sender<Frames>,
    response_receiver: Receiver<Frames>,
    w1: u32,
    w2: u32,
    h1: u32,
    h2: u32,
    unix_stream: Arc<UnixStream>,
}

impl CameraProc {
    pub fn new(unix_stream: UnixStream, w1: u32, h1: u32, w2: u32, h2: u32) -> Self {
        let (data_sender, data_receiver) = unbounded();
        let (response_sender, response_receiver) = unbounded();
        let unix_stream = Arc::new(unix_stream);

        Self {
            data_sender,
            data_receiver,
            w1,
            w2,
            h1,
            h2,
            response_sender,
            response_receiver,
            unix_stream,
        }
    }

    pub fn run(&self) -> JoinHandle<error_stack::Result<(), GError>> {
        let instance = self.clone();
        // TODO: logging
        println!("Head Detection model connected");

        let w1 = self.w1;
        let w2 = self.w2;
        let h1 = self.h1;
        let h2 = self.h2;

        thread::spawn(move || {
            instance.send_u32(w1)?;
            instance.send_u32(h1)?;

            instance.send_u32(w2)?;
            instance.send_u32(h2)?;
            loop {
                let sig = instance.recv_data()?;

                instance.send_u32(sig)?;
                let img1 = instance.recv_ipc()?;
                instance.send_u32(2)?;
                let img2 = instance.recv_ipc()?;

                let res = Frames {
                    cam1: img1,
                    cam2: img2,
                };

                instance.send_response(res)?;
            }
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

#[derive(Default, Debug, Deserialize)]
pub struct Frames {
    pub cam1: Vec<u8>,
    pub cam2: Vec<u8>,
}
