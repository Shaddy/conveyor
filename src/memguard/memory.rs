use super::iochannel::{Device};
use super::core;
use super::structs;

#[derive(Debug)]
pub struct Map<'a> {
    device: &'a Device,
    address: u64,
    size: usize,
    raw: structs::SE_MAP_VIRTUAL_MEMORY
}

impl<'a> Map<'a> {
    pub fn new(device: &'a Device, address: u64, size: usize) -> Map<'a> {
        let raw = core::map_memory(&device, address, size);

        Map {
            device: device,
            address: address,
            size: size,
            raw: raw
        }
    }
}

impl<'a> Drop for Map<'a> {
    fn drop(&mut self) {
        core::unmap_memory(self.device, self.raw);
    }
}