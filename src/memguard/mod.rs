// Copyright © ByteHeed.  All rights reserved.

extern crate clap;
extern crate slog;
extern crate winapi;
extern crate byteorder;
extern crate num;

use super::iochannel;

mod core;
pub mod command;

pub struct Guard {
    id: u64
}

pub struct Partition {
    id: u64,
    guards: Vec<Guard>,
}

impl Partition {
    pub fn new() -> Partition {
        Partition {
            id: core::create_partition().expect("Unable to create partition"),
            guards: Vec::new()
        }
    }

    pub fn remove(&mut self, guard: Guard) -> Result<(), String> {
        let (index, _) = self.guards.iter().enumerate()
        .filter(|&(_, g)| g.id == guard.id)
        .take(1)
        .nth(0)
        .unwrap();

        self.guards.remove(index);
        Ok(())
    }

    pub fn add(&mut self, guard: Guard) -> Result<(), String> {
        self.guards.push(guard);
        Ok(())
    }
}

impl Drop for Partition {
    fn drop(&mut self) {
        core::delete_partition(self.id).expect("Can't destroy patrition")
    }
}