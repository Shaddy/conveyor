use std::sync::mpsc;
use super::sync::{Event};
use super::structs::{ ObjectType,
                      OPEN_MESSAGE,
                      CLOSE_MESSAGE,
                      DELETE_MESSAGE,
                      PARSE_MESSAGE,
                      SECURITY_MESSAGE,
                      QUERYNAME_MESSAGE,
                      OKAYTOCLOSE_MESSAGE };

use std::{mem, fmt, thread, slice};

use std::fmt::Debug;
use super::{Action, Access, CallbackMap};

const BUCKET_SIZE: usize = (240 + 16);

bitflags! {
    pub struct ControlFlags: u32 {
        const SE_MESSAGE_NORMAL       = 0x0000_0000;
        const SE_MESSAGE_ASYNCHRONOUS = 0x0000_0001;
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Bucket {
    header: MessageHeader
}

#[derive(Debug)]
#[repr(C)]
pub struct Syncronizers {
    pub user: Event,
    pub kernel: Event,
}

impl Syncronizers {
    pub unsafe fn from_raw(ptr: *const u8) -> Syncronizers {
        mem::transmute_copy(&*ptr)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum MessageType {
    Unknown = 0x0000_0000_0000_0000,
    Intercept,
    Monitor,
    Terminate,
    Error,
}

#[derive(Debug)]
#[repr(C)]
struct MessageHeader {
    id: u64,
    control: ControlFlags,
    kind: MessageType
}

// impl MessageHeader {
//     pub unsafe fn from_raw(ptr: *const u8) -> MessageHeader {
//         mem::transmute_copy(&*ptr)
//     }
// }

// #[derive(Debug)]
// #[repr(C)]
// pub struct GuardedRegionAction {
//     Type: u16,
//     ReadBuffer: u64,
//     WriteBuffer: u64
// }

#[repr(C)]
pub struct FrameContext {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rdi: u64,
    rsi: u64,
    rbp: u64,
    rsp: u64,
    rbx: u64,
    rdx: u64,
    rcx: u64,
    rax: u64,
    rip: u64,
    rflags: u64
}

const MAX_INST_LENGHT: usize = 16;

#[repr(C)]
pub struct Monitor {
    header: MessageHeader,
    pub kind: ObjectType,
}

impl Monitor {
    pub unsafe fn from_raw(ptr: *const u8) -> Monitor {
        mem::transmute_copy(&*ptr)
    }

    unsafe fn get_message<T: Debug>(ptr: *const u8) -> String {
        let m =  mem::transmute_copy::<T, T> (&*(ptr as *const T));
        format!("{:?}", m)
    }
}

#[repr(C)]
pub struct Interception {
    header: MessageHeader,
    pub guard_id: u64,
    pub region_id: u64,
    pub cpu: FrameContext,
    pub instruction: [u8; MAX_INST_LENGHT],
    pub processor: u8,
    pub process: u64,
    pub address: u64,
    pub access: Access,
    pub flags: u32,
    pub context: u64,
    pub action: Action
}

impl Interception {
    pub unsafe fn from_raw(ptr: *const u8) -> Interception {
        mem::transmute_copy(&*ptr)
    }
}

impl Debug for Interception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Region(0x{:016X}) => 0x{:016x} - ({:?}) - ({:?})",
                self.region_id,
                self.address,
                self.access,
                self.action)
    }
}

#[derive(Debug)]
pub struct Response {
    message: Option<String>,
    action: Action
}

impl Response {
    pub fn action(&self) -> Action {
        self.action
    }

    pub fn message(&self) -> String {
        if let Some(ref msg) = self.message {
            return msg.clone()
        }

        String::from("")
    }

    pub fn empty() -> Response {
        Response {
            message: None,
            action: Action::CONTINUE
        }
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    pub fn new(message: Option<String>, action: Action) -> Response {
        Response {
            message: message,
            action: action
        }
    }
}

impl Bucket {
    pub fn slice_buckets(ptr: u64, capacity: usize) -> Vec<Vec<u8>> {
        let chunks = BUCKET_SIZE;

        let ptr: *mut u8 = ptr as *mut u8;

        let size = capacity / chunks;

        unsafe {
            let mut buffers: Vec<Vec<u8>> = Vec::new();

            for current in (0..capacity).step_by(BUCKET_SIZE) {
                buffers.push(Vec::from_raw_parts(ptr.offset(current as isize), size, size));
            };

            buffers
        }
    }

    fn set_action(&self, ptr: *const u8, action: Action) {
        unsafe {
            // let intercept: &mut Interception = &mut mapping.as_mut_ptr().offset(mem::size_of::<Syncronizers>() as isize) as *mut Interception;
            let intercept: &mut Interception = mem::transmute::<*const u8, &mut Interception>(ptr
                                                .offset(mem::size_of::<Syncronizers>() as isize));
            intercept.action = action;
        }
    }

    pub fn handler(messenger: mpsc::Sender<String>, mapping: Vec<u8>, default: Box<Fn(Interception) -> Response>, callbacks: CallbackMap) {
        let sync = unsafe{ Syncronizers::from_raw(mapping.as_ptr()) } ;
        // println!("#{:?} - {:?}", thread::current().id(), sync);

        // in order to prevent heapfree over false Vec reference
        // we create a reference to a slice
        let mapping = unsafe {
            let (ptr, len) = (mapping.as_ptr(), mapping.len());
            mem::forget(mapping);
            slice::from_raw_parts(ptr, len)
        };

        let id = thread::current().id();

        loop {
            // println!("#{:?} - waiting for new messsage.", thread::current().id());

            sync.kernel.wait();

            // println!("#{:?} - got bucket", thread::current().id());

            let bucket = unsafe{ Bucket::from_raw(mapping.as_ptr()
                                            // skip events
                                            .offset(mem::size_of::<Syncronizers>() as isize)) } ;

            // println!("#{:?} - parsed bucket", thread::current().id());

            let response = match bucket.header.kind {
                MessageType::Terminate => {
                    // println!("#{:?} - terminate message.", thread::current().id());
                    sync.user.signal();
                    break
                },
                MessageType::Intercept => {
                    // println!("#{:?} - redirecting interception", thread::current().id());
                    let interception = unsafe { Interception::from_raw(mapping.as_ptr()
                                    .offset(mem::size_of::<Syncronizers>() as isize)) };

                    let map = callbacks.read().expect("Unable to unlock callbacks for reading");

                    let response = match map.get(&interception.guard_id) {
                        Some(callback) => callback(interception),
                        None => default(interception)
                    };

                    bucket.set_action(mapping.as_ptr(), response.action());

                    response
                },
                MessageType::Monitor => {
                    let monitor = unsafe { Monitor::from_raw(mapping.as_ptr()
                                    .offset(mem::size_of::<Syncronizers>() as isize)) };

                    let offset = mem::size_of::<Syncronizers>() as isize +
                                 mem::size_of::<Monitor>() as isize;

                    let message = match monitor.kind {
                            ObjectType::OpenMessage => {
                                unsafe {
                                    Monitor::get_message::<OPEN_MESSAGE>
                                            (mapping.as_ptr().offset(offset))
                                }
                            },
                            ObjectType::CloseMessage => {
                                unsafe {
                                    Monitor::get_message::<CLOSE_MESSAGE>
                                            (mapping.as_ptr().offset(offset))
                                }
                            },
                            ObjectType::DeleteMessage => {
                                unsafe {
                                    Monitor::get_message::<DELETE_MESSAGE>
                                            (mapping.as_ptr().offset(offset))
                                }
                            },
                            ObjectType::ParseMessage => {
                                unsafe {
                                    Monitor::get_message::<PARSE_MESSAGE>
                                            (mapping.as_ptr().offset(offset))
                                }
                            },
                            ObjectType::SecurityMessage => {
                                unsafe {
                                    Monitor::get_message::<SECURITY_MESSAGE>
                                            (mapping.as_ptr().offset(offset))
                                }
                            },
                            ObjectType::QueryNameMessage => {
                                unsafe {
                                    Monitor::get_message::<QUERYNAME_MESSAGE>
                                            (mapping.as_ptr().offset(offset))
                                }
                            },
                            ObjectType::OkayToCloseMessage => {
                                unsafe {
                                    Monitor::get_message::<OKAYTOCLOSE_MESSAGE>
                                            (mapping.as_ptr().offset(offset))
                                }
                            }
                    };

                    Response::new(Some(message), Action::CONTINUE)

                }
                _ => { Response::empty() }
            };

            // println!("#{:?} message-ready, notifying back.", thread::current().id());

            // TODO: Convert this check into something more fancy.
            if (bucket.header.control & ControlFlags::SE_MESSAGE_ASYNCHRONOUS) != ControlFlags::SE_MESSAGE_ASYNCHRONOUS {
                sync.user.signal();
            }

            if response.has_message() {
                let message = format!("LAST-EVENT: {:?}", response.message());

                if let Err(err) = messenger.send(message) {
                    panic!("error sending to messenger: {}", err.to_string());
                }
            }
        }
    }

    pub unsafe fn from_raw(ptr: *const u8) -> Bucket {
        mem::transmute_copy(&*ptr)
    }

}

// DEPRECATED DUE TO mem::transmute, just keeping it until all tests are guaranteed.
//
//
// impl Into<Vec<u8>> for Bucket {
//     fn into(self) -> Vec<u8> {
//         let mut v = Vec::new();
//         let _ = v.write_u64::<LittleEndian>(self.user.into()).unwrap();
//         let _ = v.write_u64::<LittleEndian>(self.kernel.into()).unwrap();
//         let _ = v.write(&self.data).unwrap();

//         v
//     }
// }

// impl<'a> From<&'a Vec<u8>> for Bucket {
//     fn from(v: &Vec<u8>) -> Bucket {
//         let mut cursor = Cursor::new(v);
//         let mut bucket = Bucket {
//             user: Event::from(cursor.read_u64::<LittleEndian>().unwrap()),
//             kernel: Event::from(cursor.read_u64::<LittleEndian>().unwrap()),
//             data: [0; 240]
//         };

//         cursor.read(&mut bucket.data).unwrap();

//         bucket
//     }
// }
// impl From<Vec<u8>> for Bucket {
//     fn from(v: Vec<u8>) -> Bucket {
//         let mut cursor = Cursor::new(v);
//         let mut bucket = Bucket {
//             user: Event::from(cursor.read_u64::<LittleEndian>().unwrap()),
//             kernel: Event::from(cursor.read_u64::<LittleEndian>().unwrap()),
//             data: [0; 240]
//         };

//         cursor.read(&mut bucket.data).unwrap();

//         bucket
//     }
// }
