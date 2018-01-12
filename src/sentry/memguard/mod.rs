use super::iochannel::{Device};
mod bucket;
mod sync;

use super::{io, memory, misc};

use std::{fmt, thread};
use std::thread::{JoinHandle};

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

pub use self::bucket::Interception;

pub use super::structs::MatchType;

use super::failure::Error;

use super::structs::{FieldKey, 
                    ValueType, 
                    MG_GUARD_CONDITION, 
                    MG_GUARD_FILTER, 
                    MG_FIELD_VALUE};

const _PARTITION_ROOT_ID: u64 = 4;

pub enum ControlGuard {
    Start = 1,
    Stop
}

bitflags! {
    pub struct Action: u16 {
        const NOTIFY    = 0x0000_1000;
        const CONTINUE  = 0x0000_0001;
        const BLOCK     = 0x0000_0002;
        const STEALTH   = 0x0000_0004;
        const INSPECT   = 0x0000_1008;
    }
}

bitflags! {
    pub struct GuardFlags: u32 {
        const STARTED      = 0x0000_0000;
        const STOPPED      = 0x0000_0001;
    }
}

pub enum RegionStatus {
    Enable = 1,
    Disable
}

bitflags! {
    pub struct RegionFlags: u32 {
        const ENABLED    = 0x0000_0000;
        const DISABLED   = 0x0000_0001;
    }
}

bitflags! {
    pub struct Access: u16 {
        const READ       = 0x0000_0001;
        const WRITE      = 0x0000_0002;
        const EXECUTE    = 0x0000_0004;
    }
}

impl Access {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

pub struct Partition {
    pub id: u64,
    pub device: Device,
    workers: Vec<JoinHandle<()>>,
    callbacks: CallbackMap
}

impl fmt::Debug for Partition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Partition(0x{:016x})", self.id)
    }
}

pub type SyncCallback = Box<Fn(bucket::Interception) -> Action + Send + Sync>;
pub type CallbackMap = Arc<RwLock<HashMap<u64, SyncCallback>>>;

impl Partition
 {
    fn default_callback(interception: bucket::Interception) -> Action {
        println!("{:?}", interception);
        Action::CONTINUE
    }

    pub fn register_callback(&self, guard: &Guard, callback: SyncCallback) {
        let mut map = self.callbacks.write().expect("Failed to unlock as a writer");
        map.insert(guard.id, callback);
    }

    fn create_workers(&self, buckets: Vec<Vec<u8>>, 
                             callbacks: &CallbackMap) -> Vec<JoinHandle<()>> {
        buckets.into_iter().map(|bucket| 
        {
            let callbacks = Arc::clone(callbacks);
            thread::spawn(move|| bucket::Bucket::handler(bucket, Box::new(Partition::default_callback), callbacks))

        }).collect()
    }

    pub fn new() -> Result<Partition, Error> {
        let device = Device::new(io::SE_NT_DEVICE_NAME)?;
        let channel = io::create_partition(&device)?;
        let callbacks = Arc::new(RwLock::new(HashMap::new()));

        let mut partition = Partition {
            id: channel.id,
            callbacks: Arc::clone(&callbacks),
            device: device,
            workers: Vec::new()
        };
        
        let workers = partition.create_workers(
            bucket::Bucket::slice_buckets(channel.address, channel.size as usize),
            &callbacks
        );

        partition.workers.extend(workers.into_iter());

        Ok(partition)
    
    }

    pub fn root() -> Partition {
        Partition::new().unwrap()
    }

}

impl Drop for Partition {
    fn drop(&mut self) {
        if let Err(err) = io::delete_partition(&self.device, self.id) {
            println!("io::delete_partition() {}", err);
        }

        while let Some(handle) = self.workers.pop() {
             handle.join().expect("failed to wait for thread");
        }
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
    pub fn region(partition: &'p Partition, base: u64, limit: u64, action: Option<Action>, access: Access) -> Result<Sentinel<'p>, Error> {
        let range = Range::new(base, limit);

        let action = action.unwrap_or(Action::INSPECT | Action::NOTIFY);

        let id = io::create_region(&partition.device, partition.id, &range, action, access, Some(0x100))?;

        Ok(
            Sentinel::Region{
                id: id,
                partition: partition,
                range: range,
                access: access,
                action: action
        })
    }

    pub fn patch() -> Sentinel<'p> {
        Sentinel::Patches
    }

    pub fn tracepoint(base: u64, limit: u64) -> Sentinel<'p> {
        Sentinel::Tracepoint(Range { base: base, limit: limit})
    }

    pub fn remove(&self, guard: &Guard) -> Result<(), Error> {
        match *self {
            Sentinel::Region{id: region_id, .. } => {
                io::remove_region(&guard.partition.device, guard.id, region_id)?;
            },
            Sentinel::Tracepoint(_) | Sentinel::Patches => {
                // io::remove_sentinel(guard.id, trace_id)
                println!("not implemented");
            },
        }

        Ok(())
    }

    pub fn register(&self, guard: &Guard) -> Result<(), Error> {
        match *self {
            Sentinel::Region{
                id: region_id,
                ..
            } => {
                io::add_region(&guard.partition.device, guard.id, region_id)?;
            },
            Sentinel::Tracepoint(_) | Sentinel::Patches =>  {
                // io::add_sentinel(&self.partition.device, guard.id, trace_id)
                println!("not implemented");
            }
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
                id,
                ref range,
                ..
            } => {
                write!(f, "Region(id: 0x{:08X}, base: 0x{:08X} limit: 0x{:X})", id, range.base, range.limit)
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
            Sentinel::Region { id, partition, ..} => io::delete_region(&partition.device, id).unwrap(),
            Sentinel::Tracepoint { .. } => (),
            _ => {}
        };
    }
}

#[derive(Debug)]
pub struct Filter<'a> {
    pub alloc: memory::KernelAlloc<'a, MG_GUARD_FILTER>,
    pub filter: &'a mut MG_GUARD_FILTER
}

impl<'a> Filter<'a> {
    pub fn new(device: &'a Device) -> Filter {
        let alloc = memory::KernelAlloc::new(device);

        let filter = unsafe { &mut *alloc.as_mut_ptr() };

        Filter {
            alloc: alloc,
            filter: filter,
        }
    }

    pub fn kernel_ptr(&self) -> u64 {
        self.alloc.kernel_ptr()
    }

    pub fn add(&mut self, condition: &Condition) {
        let current = &mut self.filter.Conditions[self.filter.NumberOfConditions as usize];

        assert!(self.filter.NumberOfConditions < 16);

        current.Field = condition.condition.Field;
        current.Match = condition.condition.Match;
        current.Value.Kind = condition.condition.Value.Kind;
        current.Value.Value = condition.condition.Value.Value;

        self.filter.NumberOfConditions += 1;

    }

    pub fn process(device: &'a Device, name: &str, cmp: MatchType) -> Option<Filter<'a>> {
        let mut filter = Filter::new(device);

        let current = misc::WalkProcess::iter().find(|p| p.name().contains(name))
                                        .expect("process not found");

        filter.add(&Condition::new(FieldKey::PROCESS_ID, 
                                  cmp,
                                  ValueType::UINT64,
                                  current.id()));
        
        Some(filter)
    }

    pub fn current_process(device: &'a Device, cmp: MatchType) -> Option<Filter<'a>> {
        Filter::process(device, "conveyor.exe", cmp)
    }
}

#[derive(Debug)]
pub struct Condition {
    pub condition: MG_GUARD_CONDITION
}

impl Condition {
    pub fn new(field: FieldKey, cmp: MatchType, kind: ValueType, value: u64) -> Condition {
        Condition {
            condition: MG_GUARD_CONDITION {
                Field: field,
                Match: cmp,
                Value: MG_FIELD_VALUE {
                    Kind: kind,
                    Value: value
                }
            }
        }
    }
}

pub struct Guard<'p> {
    id: u64,
    partition: &'p Partition,
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
    pub fn new(partition: &'p Partition, filter: Option<Filter<'p>>) -> Guard<'p> {
        let id = io::register_guard(&partition.device, partition.id, filter)
            .expect("Unable to connect guard with root partition");

        println!("Guard<0x{:08X}>::new()", id);

        Guard {
            id: id,
            partition: partition,
            sentinels: Vec::new()
        }
    }

    pub fn start(&self) -> &Self {
        io::start_guard(&self.partition.device, self.id)
                            .expect("start error");

        self
    }

    pub fn stop(&self) -> &Self {
        io::stop_guard(&self.partition.device, self.id)
                            .expect("stop error");
        self
    }

    #[allow(dead_code, unused_variables)]
    pub fn remove(&mut self, sentinel: &Sentinel) {
        unimplemented!()
        // sentinel.unregister().expect(format!("Unable to register {:?}", sentinel));
        // self.sentinels.remove(sentinel)

    }

    pub fn set_callback(&self, callback: SyncCallback) {
        self.partition.register_callback(self, callback)
    }

    pub fn add(&mut self, sentinel: Sentinel<'p>) {
        sentinel.register(self).expect(format!("Unable to register {}", sentinel).as_ref());
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
        if let Err(err) = io::unregister_guard(&self.partition.device, self.id) {
            println!("error unregistering guard: {}", err);
        }
    }
}