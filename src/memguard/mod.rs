// Copyright © ByteHeed.  All rights reserved.

extern crate clap;
extern crate slog;
extern crate winapi;
extern crate byteorder;
extern crate num;

use super::iochannel;
use super::iochannel::{Device};

use std::fmt;
use std::thread;
use std::thread::{JoinHandle};

mod core;
mod sync;
mod bucket;
mod tests;
pub mod command;

const PARTITION_ROOT_ID: u64 = 4;

pub enum ControlGuard {
    Start = 1,
    Stop
}

bitflags! {
    pub struct Action: u16 {
        const NOTIFY    = 0x00001000;
        const CONTINUE  = 0x00000001;
        const BLOCK     = 0x00000002;
        const STEALTH   = 0x00000003;
        const INSPECT   = 0x00001008;
    }
}

bitflags! {
    pub struct GuardFlags: u32 {
        const STARTED      = 0x00000000;
        const STOPPED      = 0x00000001;
    }
}

pub enum RegionStatus {
    Enable = 1,
    Disable
}

bitflags! {
    pub struct RegionFlags: u32 {
        const ENABLED    = 0x00000000;
        const DISABLED   = 0x00000001;
    }
}

bitflags! {
    pub struct Access: u16 {
        const READ       = 0x00000001;
        const WRITE      = 0x00000002;
        const EXECUTE    = 0x00000004;
    }
}

impl Access {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

#[derive(Debug)]
pub struct Partition {
    pub id: u64,
    pub device: Device,
    workers: Vec<JoinHandle<()>>
}

impl Partition
 {
    fn callback(interception: bucket::Interception) -> Action {
        println!("[!] {:?}", interception);
        Action::CONTINUE
    }

    // pub fn register_callback(&mut self, id: u64, callback: &Fn(bucket::Interception) -> Action) {
    //     self.callbacks.insert(id, callback);
    // }



    fn create_workers(&self, buckets: Vec<Vec<u8>>) -> Vec<JoinHandle<()>> {
        buckets.into_iter().map(|bucket| 
        {
            thread::spawn(move|| bucket::Bucket::handler(bucket, &Partition::callback))

        }).collect()
    }

    pub fn new() -> Partition {
        let device = Device::new(core::SE_NT_DEVICE_NAME);
        let channel = core::create_partition(&device).expect("Unable to create partition");

        println!("Partition::new() => channel: {:?}", channel);

        let mut partition = Partition {
            id: channel.id,
            device: device,
            workers: Vec::new()
        };
        
        let workers = partition.create_workers(
            bucket::Bucket::slice_buckets(channel.address, channel.size as usize)
        );

        partition.workers.extend(workers.into_iter());


        partition
    
    }

    fn root() -> Partition {
        Partition::new()
    }
}

impl Drop for Partition {
    fn drop(&mut self) {
        println!("deleting partition");
        core::delete_partition(&self.device, self.id).expect("Can't destroy partition");
        // self.workers.iter().for_each(|ref worker| worker.join().unwrap());
    }
}


#[derive(Debug)]
pub struct Range {
    pub base: u64,
    pub limit: u64
}

impl Range {
    pub fn new(base: u64, limit: u64) -> Range {
        Range {
            base: base,
            limit: limit
        }
    }
}

#[derive(Debug)]
pub enum Sentinel<'p> {
    Region {
        id: u64,
        partition: &'p Partition,
        range: Range,
        access: Access,
        action: Action
    },
    Tracepoint(Range),
    Patches
}

impl<'p> Sentinel<'p> {
    pub fn region(partition: &'p Partition, base: u64, limit: u64, access: Access) -> Sentinel {
        let range = Range::new(base, limit);
        let id = core::create_region(&partition.device, partition.id, &range, access, Some(0x100));

        Sentinel::Region{
            id: id,
            partition: partition,
            range: range,
            access: Access::READ,
            action: Action::CONTINUE
        }
    }

    pub fn patch() -> Sentinel<'p> {
        Sentinel::Patches
    }

    pub fn tracepoint(base: u64, limit: u64) -> Sentinel<'p> {
        Sentinel::Tracepoint(Range { base: base, limit: limit})
    }

    pub fn remove(&self, guard: &Guard) -> Result<(), ()> {
        match *self {
            Sentinel::Region{
                id: region_id,
                partition: _, 
                range: _,
                access: _,
                action: _
            } => {
                core::remove_region(&guard.partition.device, guard.id, region_id);
            },
            Sentinel::Tracepoint(_) =>  {
                // core::remove_tracepoint(guard.id, trace_id)
            },
            Sentinel::Patches => {
                // core::remove_patch(guard.id, trace_id)
            },
        }

        Ok(())
    }

    pub fn register(&self, guard: &Guard) -> Result<(), ()> {
        match *self {
            Sentinel::Region{
                id: region_id,
                partition: _, 
                range: _,
                access: _,
                action: _
            } => {
                core::add_region(&guard.partition.device, guard.id, region_id);
            },
            Sentinel::Tracepoint(_) =>  {
                // core::add_tracepoint(&self.partition.device, guard.id, trace_id)
            },
            Sentinel::Patches => {
                // core::add_patch(&self.partition.device, guard.id, trace_id)
            },
        }

        Ok(())
    }
}

// DEBUG IMPL for Sentinels
// Derived is enough, but in future we should expand this
//
// impl<'p> fmt::Debug for Sentinel<'p> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             Sentinel::Region{
//                 id: region_id,
//                 partition: _, 
//                 ref range,
//                 access: _,
//                 action: _
//             } => {
//                 write!(f, "Region(id: 0x{:08X}, base: 0x{:08X} limit: 0x{:X})", region_id, range.base, range.limit)
//             },
//             Sentinel::Tracepoint(_) =>  {
//                 write!(f, "Tracepoint")
//             },
//             Sentinel::Patches => {
//                 write!(f, "Patch")
//             },
//         }

//     }
// }

impl<'p> fmt::Display for Sentinel<'p> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Sentinel::Region{
                id: region_id,
                partition: _, 
                ref range,
                access: _,
                action: _
            } => {
                write!(f, "Region(id: 0x{:08X}, base: 0x{:08X} limit: 0x{:X})", region_id, range.base, range.limit)
            },
            Sentinel::Tracepoint(_) =>  {
                write!(f, "Tracepoint")
            },
            Sentinel::Patches => {
                write!(f, "Patch")
            },
        }

    }
}


impl<'p> Drop for Sentinel<'p> {
    fn drop(&mut self) {
        match *self {
            Sentinel::Region {
                id: region_id,
                partition, 
                range: _,
                access: _,
                action: _
            } => core::delete_region(&partition.device, region_id),
            _ => {}
        };
    }
}

#[derive(Debug)]
pub enum Filter {
    None
}

// TODO: Add user callback to notify interceptions.

pub struct Guard<'p> {
    id: u64,
    _filter: Filter,
    partition: &'p Partition,
    // callback: Fn(&bucket/whatever),
    sentinels: Vec<Sentinel<'p>>,
}

// a generic intention of having a commmon partition creator attached
// to contained objects
//
// trait Partitioned {
//     fn root_partition() -> Rc<Partition> {
//         Rc::new(Partition::from(PARTITION_ROOT_ID)
//               .unwrap_or(Partition::new()))
//     }
// }

// impl Partitioned for Guard<'p> {}
// impl<'p> Partitioned for Sentinel<'p> {}

impl<'p> Guard<'p> {
    pub fn new(partition: &'p Partition) -> Guard {

        let id = core::register_guard(&partition.device, partition.id)
            .expect("Unable to connect guard with root partition");

        println!("Guard<0x{:08X}>::new()", id);
        Guard {
            id: id,
            _filter: Filter::None,
            partition: partition,
            sentinels: Vec::new()
        }
    }

    // pub fn register_callback<'a>(callback: Fn(&whatever)) -> &'a Self{
    //      self.callback = callback      // single-callback
    // }

    pub fn start<'a>(&'a self) -> &'a Self{
        core::start_guard(&self.partition.device, self.id);

        self
    }

    pub fn stop<'a>(&'a self) -> &'a Self{
        core::stop_guard(&self.partition.device, self.id);
        self
    }

    pub fn remove(&mut self, _sentinel: Sentinel) {
        unimplemented!()
        // sentinel.unregister().expect(format!("Unable to register {:?}", sentinel));
        // self.sentinels.remove(sentinel)

    }

    pub fn add(&mut self, sentinel: Sentinel<'p>) {
        sentinel.register(&self).expect(format!("Unable to register {}", sentinel).as_ref());
        self.sentinels.push(sentinel)
    }

}


impl<'p> fmt::Display for Guard<'p> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Guard(id: {:08X})", self.id)
    }
}

impl<'p> Drop for Guard<'p> {
    fn drop(&mut self) {
        println!("Guard<0x{:08X}>::drop()", self.id);
        core::unregister_guard(&self.partition.device, self.id);
    }
}