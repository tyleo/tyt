use crate::{Error, Result};
use std::sync::mpsc;
use wgpu::{Buffer, Device, MapMode, PollType};

/// Maps `readback` for reading once the GPU has written it.
pub fn map_readback(device: &Device, readback: &Buffer) -> Result<()> {
    let (sender, receiver) = mpsc::channel();

    readback.map_async(MapMode::Read, .., move |result| {
        sender.send(result).expect("the receiver outlives the wait");
    });

    device
        .poll(PollType::wait_indefinitely())
        .map_err(Error::Poll)?;

    receiver
        .recv()
        .expect("the map callback runs during the wait")
        .map_err(Error::Map)
}
