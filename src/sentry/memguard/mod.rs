extern crate byteorder;
extern crate winapi;
extern crate console;

use super::iochannel::{Device};
use super::cli::output::{create_messenger, ShellMessage, MessageType};

use std::sync::mpsc;

mod bucket;
mod sync;
mod structs;

use self::console::style;
use super::{io, memory, misc};
use std::rc::{Rc, Weak};

use std::{fmt, thread, time};
use std::thread::{JoinHandle};

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

pub use self::bucket::{Interception, Response};

pub use self::structs::MatchType;

use super::failure::Error;

use self::structs::{FieldKey,
                    ValueType,
                    MG_GUARD_CONDITION,
                    MG_GUARD_FILTER,
                    MG_FIELD_VALUE};

const _PARTITION_ROOT_ID: u64 = 4;
pub const MESSENGER_FINISH_MSG: &str = "END-LOOP-MSG";


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

pub type SyncCallback = Box<dyn Fn(bucket::Interception) -> Response + Send + Sync>;
pub type CallbackMap = Arc<RwLock<HashMap<u64, SyncCallback>>>;

#[derive(Debug)]
pub enum Handler {
    Interceptor(JoinHandle<()>),
    Messenger(JoinHandle<()>)
}

struct Tunnel {
    workers: Vec<Handler>,
    messenger: mpsc::Sender<String>,
    callbacks: CallbackMap
}

impl Tunnel {
    fn default_callback(_interception: bucket::Interception) -> Response {
        Response::new(Some(String::from("default-callback()")), Action::CONTINUE)
    }

    pub fn register_callback(&self, guard: &Guard, callback: SyncCallback) {
        let mut map = self.callbacks.write().expect("Failed to unlock as a writer");
        map.insert(guard.id, callback);
    }

    fn create_workers(&self,
                      tx: mpsc::Sender<String>,
                      rx: mpsc::Receiver<String>,
                      buckets: Vec<Vec<u8>>,
                      callbacks: &CallbackMap) -> Vec<Handler> {

        let mut handlers = buckets.into_iter().map(|bucket|
        {
            let callbacks = Arc::clone(callbacks);
            let sender = tx.clone();
            Handler::Interceptor(
                thread::spawn(move|| bucket::Bucket::handler(sender,
                                        bucket,
                                        Box::new(Tunnel::default_callback),
                                        callbacks)
                 )
            )

        }).collect::<Vec<Handler>>();


        let (messenger, recv) = mpsc::channel();
        let handler = create_messenger(recv, Some(time::Duration::from_millis(1)), 0);

        handlers.push(Handler::Messenger(thread::spawn(move|| {
            loop {
                let message = rx.recv().unwrap();
                if message.contains(MESSENGER_FINISH_MSG) {
                    ShellMessage::send(&messenger, "- FINISHED -".to_string(),
                                                MessageType::Spinner, 0);
                    ShellMessage::send(&messenger, "- FINISHED -".to_string(),
                                                MessageType::Close, 0);
                    ShellMessage::send(&messenger, "- FINISHED -".to_string(),
                                                MessageType::Exit, 0);
                    handler.join()
                           .expect("unable to wait for shell messenger");
                    break;
                }

                ShellMessage::send(&messenger, format!("{}",
                                              style(message).magenta()),
                                              MessageType::Spinner,
                                              0);
            }
        })));

        handlers
    }

    pub fn new(channel: &io::Channel) -> Result<Tunnel, Error> {
        let callbacks = Arc::new(RwLock::new(HashMap::new()));

        let (tx, rx) = mpsc::channel();

        let mut tunnel = Tunnel {
            callbacks: Arc::clone(&callbacks),
            messenger: tx.clone(),
            workers: Vec::new()
        };

        let workers = tunnel.create_workers(
            tx,
            rx,
            bucket::Bucket::slice_buckets(channel.address, channel.size as usize),
            &callbacks
        );

        tunnel.workers.extend(workers.into_iter());

        Ok(tunnel)

    }

    fn close_workers(&mut self) {
        let mut handler: Option<JoinHandle<()>> = None;

        while let Some(handle) = self.workers.pop() {
             match handle {
                 Handler::Messenger(messenger) => {
                     handler = Some(messenger);
                 },
                 Handler::Interceptor(interceptor) => {
                 interceptor.join()
                            .expect("wait error for interceptor");
                 }
             }
        }

        self.messenger.send(MESSENGER_FINISH_MSG.to_string())
                    .expect("error finishing displayer thread");

        if let Some(handle) = handler {
            handle.join().expect("wait error for messenger");
        }
    }

}

impl Drop for Tunnel {
    fn drop(&mut self) {
        self.close_workers();
    }
}

pub struct ObjectFilter {
    pub id: u64,
    _tunnel: Tunnel,
    pub device: Rc<Device>,
}

impl ObjectFilter {
    pub fn new() -> Result<ObjectFilter, Error> {
        let device = Rc::new(Device::new(io::SE_NT_DEVICE_NAME).expect("sentry device"));
        let channel = io::create_monitor(&device)?;
        let tunnel = Tunnel::new(&channel)?;

        Ok(
            ObjectFilter {
                id: channel.id,
                device: Rc::clone(&device),
                _tunnel: tunnel
            }
        )
    }

    pub fn start(&self) -> Result<(), Error> {
        Ok(io::start_monitor(&self.device, self.id)?)
    }

    pub fn stop(&self) -> Result<(), Error> {
        Ok(io::stop_monitor(&self.device, self.id)?)
    }
}


impl Drop for ObjectFilter {
    fn drop(&mut self) {
        if let Err(err) = io::destroy_monitor(&self.device, self.id) {
            println!("io::destroy_monitor() {}", err);
        }
    }
}

impl fmt::Debug for ObjectFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ObjectFilter(0x{:016x})", self.id)
    }
}

pub struct Partition {
    pub id: u64,
    pub device: Rc<Device>,
    tunnel: Tunnel,
}

impl Partition
{
    pub fn new() -> Result<Partition, Error> {
        let device = Rc::new(Device::new(io::SE_NT_DEVICE_NAME).expect("sentry device"));
        let channel = io::create_partition(&device)?;
        let tunnel = Tunnel::new(&channel)?;

        Ok(
            Partition {
                id: channel.id,
                device: Rc::clone(&device),
                tunnel: tunnel
            }
        )
    }

    pub fn register_callback(&self, guard: &Guard, callback: SyncCallback) {
        self.tunnel.register_callback(guard, callback)
    }

    pub fn device(&self) -> Weak<Device> {
        Rc::downgrade(&self.device)
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
    }
}

impl fmt::Debug for Partition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Partition(0x{:016x})", self.id)
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

pub trait Sentinel {
    fn remove(&self, guard: &Guard) -> Result<(), Error>;
    fn register(&self, guard: &Guard) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct Region<'p> {
    id: u64,
    partition: &'p Partition,
    range: Range,
    access: Access,
    action: Action
}

impl<'p> Region<'p> {
    pub fn new(partition: &'p Partition, base: u64, limit: u64, action: Option<Action>, access: Access) -> Result<Region<'p>, Error> {
        let range = Range::new(base, limit);

        let action = action.unwrap_or(Action::INSPECT | Action::NOTIFY);

        let id = io::create_region(&partition.device, partition.id, &range, action, access, Some(0x100))?;

        Ok(
            Region {
                id: id,
                partition: partition,
                range: range,
                access: access,
                action: action
        })
    }
}

impl<'p> fmt::Display for Region<'p> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Region(id: 0x{:08X}, base: 0x{:08X} limit: 0x{:X})",
                        self.id,
                        self.range.base,
                        self.range.limit)
    }
}

impl<'p> Sentinel for Region<'p> {
    fn remove(&self, guard: &Guard) -> Result<(), Error> {
        io::remove_region(&self.partition.device, guard.id, self.id)
    }

    fn register(&self, guard: &Guard) -> Result<(), Error> {
        io::add_region(&self.partition.device, guard.id, self.id)
    }
}

#[derive(Debug)]
pub struct Patch<'p> {
    id: u64,
    partition: &'p Partition,
    base: u64,
    patch: u64,
    limit: u64
}

impl<'p> Patch<'p> {
    pub fn new(partition: &'p Partition, base: u64, patch: u64, limit: u64) -> Result<Patch<'p>, Error> {
        let patch_range = Range::new(patch, limit);

        let id = io::create_patch(&partition.device, partition.id, base, &patch_range)?;

        Ok(
            Patch {
                id: id,
                partition: partition,
                base: base,
                patch: patch,
                limit: limit
        })

    }
}

impl<'p> fmt::Display for Patch<'p> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Patch(id: 0x{:08X}, base: 0x{:08X} patch: 0x{:08X} limit: 0x{:X} )",
                        self.id,
                        self.base,
                        self.patch,
                        self.limit)
    }
}

impl<'p> Sentinel for Patch<'p> {
    fn remove(&self, guard: &Guard) -> Result<(), Error> {
        io::remove_patch(&self.partition.device, guard.id, self.id)
    }

    fn register(&self, guard: &Guard) -> Result<(), Error> {
        io::add_patch(&self.partition.device, guard.id, self.id)
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
        if let Some(current) = misc::WalkProcess::iter().find(|p| p.name().contains(name)) {
            let mut filter = Filter::new(device);
            filter.add(&Condition::new(FieldKey::PROCESS_ID,
                                    cmp,
                                    ValueType::UINT64,
                                    current.id()));

            return Some(filter)
        }

        None
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
    // sentinels: Vec<Sentinel>,
}

impl<'p> Guard<'p> {
    pub fn new(partition: &'p Partition, filter: Option<Filter<'p>>) -> Guard<'p> {
        let id = io::register_guard(&partition.device, partition.id, filter)
            .expect("Unable to connect guard with root partition");

        Guard {
            id: id,
            partition: partition,
            // sentinels: Vec::new()
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

    pub fn remove<T>(&mut self, sentinel: T) where T:
        Sentinel + fmt::Display {
        sentinel.remove(self).expect(format!("Unable to register {}", sentinel).as_ref());
    }

    pub fn set_callback(&self, callback: SyncCallback) {
        self.partition.register_callback(self, callback)
    }

    pub fn add<T>(&mut self, sentinel: T) where T:
        Sentinel + fmt::Display {
        sentinel.register(self).expect(format!("Unable to register {}", sentinel).as_ref());
    }
}


impl<'p> fmt::Display for Guard<'p> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Guard(id: {:08X})", self.id)
    }
}

impl<'p> Drop for Guard<'p> {
    fn drop(&mut self) {
        if let Err(err) = io::unregister_guard(&self.partition.device, self.id) {
            println!("error unregistering guard: {}", err);
        }
    }
}
