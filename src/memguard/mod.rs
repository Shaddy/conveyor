// Copyright © ByteHeed.  All rights reserved.

extern crate clap;
extern crate slog;
extern crate winapi;
extern crate byteorder;
extern crate num;

use super::iochannel;

use std::fmt;
use std::thread;
use std::thread::{JoinHandle};

mod core;
mod sync;
mod bucket;
mod tests;
pub mod command;

const PARTITION_ROOT_ID: u64 = 4;

bitflags! {
    pub struct Status: u32 {
        const DISABLED   = 0x00000000;
        const ENABLED    = 0x00000001;
    }
}

bitflags! {
    pub struct Access: u32 {
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
    workers: Vec<JoinHandle<()>>
}

// fn sleep(seconds: u64) {
//     let duration = std::time::Duration::from_secs(seconds);
//     println!("sleeping: {} seconds.", duration.as_secs());
//     thread::sleep(duration);
//     println!("done");
// }

impl Partition
 {
    fn create_workers(buckets: Vec<Vec<u8>>) -> Vec<JoinHandle<()>> {
        buckets.into_iter().map(|bucket| 
        {
            thread::spawn(move|| bucket::Bucket::handler(bucket))

        }).collect()
    }

    pub fn new() -> Partition {
        println!("new partition");

        let channel = core::create_partition().expect("Unable to create partition");
        
        let workers = Partition::create_workers(
            bucket::Bucket::slice_buckets(channel.address, channel.size)
        );

        Partition {
            id: channel.id,
            workers: workers
        }
    
    }

    fn root() -> Partition {
        Partition::from(PARTITION_ROOT_ID)
              .unwrap_or(Partition::new())
    }

    pub fn from(id: u64) -> Result<Partition, String> {

        // TODO: Improve a mechanism to discover an existing partition.
        if let Err(err) = core::get_partition_option(id, 1) {
            match err {
                core::PartitionError::NotExists => {
                    return Err("partition doesn't exist".to_string())
                },
                _                               => {
                    panic!("unknown-partition-error: {:?}", err)
                }
            }
        } else {
            println!("Partition already exists, creating his object.");
            Ok(Partition { id: id,
             workers: Vec::new()
             })
        }

    }
}

impl Drop for Partition {
    fn drop(&mut self) {
        println!("deleting partition");
        core::delete_partition(self.id).expect("Can't destroy patrition")
    }
}

#[derive(Debug)]
pub enum Action {
    None,
    Notify,
    Continue,
    Block,
    Stealth,
    Inspect
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
        let id = core::create_region(partition.id, &range, access, None);

        
        Sentinel::Region{
            id: id,
            partition: partition,
            range: range,
            access: Access::READ,
            action: Action::None
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
                core::remove_region(guard.id, region_id);
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
                core::add_region(guard.id, region_id);
            },
            Sentinel::Tracepoint(_) =>  {
                // core::add_tracepoint(guard.id, trace_id)
            },
            Sentinel::Patches => {
                // core::add_patch(guard.id, trace_id)
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
                partition: _, 
                range: _,
                access: _,
                action: _
            } => core::delete_region(region_id),
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
    _partition: &'p Partition,
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

        let id = core::register_guard(partition.id)
            .expect("Unable to connect guard with root partition");

        println!("Guard<0x{:08X}>::new()", id);
        Guard {
            id: id,
            _filter: Filter::None,
            _partition: partition,
            sentinels: Vec::new()
        }
    }

    // fn handler(buffer: mut Vec<u8>) -> {
    //     let mut bucket = Bucket::from(buffer); 
    //     bucket.kernel.wait();

    //    // dispatch interception code
    //     match bucket.message_type() {
    //         Bucket::Unknown(_) => (),
    //         Bucket::Interception(guard) => (),
    //         Bucket::Terminating => (),
    //     }

    //     // mutating vector example as response
    //     bucket.set_action(BucketAction::Deny)
    //     bucket.user.signal()
    // }
    
    // one guard could have multiple callbacks?
    // pub fn register_callback<'a>(callback: Fn(&whatever)) -> &'a Self{
    //      self.callback = callback      // single-callback
    //      ///////////////////////////////////////////////
    //      self.callbacks.push(callback) // multi-callback
    // }

    pub fn start<'a>(&'a self) -> &'a Self{
        core::start_guard(self.id);

        self
    }

    pub fn stop<'a>(&'a self) -> &'a Self{
        core::stop_guard(self.id);
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
        core::unregister_guard(self.id);
    }
}