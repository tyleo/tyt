use wgpu::{
    Buffer, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

/// A storage buffer of `words`, with zeros up to `least` words where it
/// holds fewer. A binding cannot be empty, and a shader's array needs room
/// for one element.
pub fn storage_words(device: &Device, label: &str, words: &[u32], least: usize) -> Buffer {
    let padded;

    let words = if words.len() < least {
        padded = [words, &vec![0; least - words.len()]].concat();

        &padded
    } else {
        words
    };

    device.create_buffer_init(&BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(words),
        usage: BufferUsages::STORAGE,
    })
}
