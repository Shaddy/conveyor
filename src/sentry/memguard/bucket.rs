extern crate byteorder;
extern crate winapi;

use super::sync::Event;

use std::mem;

use std::fmt::Debug;
use std::fmt;
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

    fn set_action(&self, buffer: &mut Vec<u8>, action: Action) {
        unsafe {
            // let intercept: &mut Interception = &mut buffer.as_mut_ptr().offset(mem::size_of::<Syncronizers>() as isize) as *mut Interception;
            let intercept: &mut Interception = mem::transmute::<*mut u8, &mut Interception>(buffer.as_mut_ptr()
                                                .offset(mem::size_of::<Syncronizers>() as isize));
            intercept.action = action;
        }
    }

    pub fn handler(mut buffer: Vec<u8>, default: Box<Fn(Interception) -> Action>, callbacks: CallbackMap) {
        let sync = unsafe{ Syncronizers::from_raw(buffer.as_ptr()) } ;
        // println!("#{:?} - {:?}", thread::current().id(), sync);

        loop {
            // println!("#{:?} - waiting for new messsage.", thread::current().id());

            sync.kernel.wait();

            // println!("#{:?} - got bucket", thread::current().id());

            let bucket = unsafe{ Bucket::from_raw(buffer.as_mut_ptr()
                                            // skip events
                                            .offset(mem::size_of::<Syncronizers>() as isize)) } ;

            // println!("#{:?} - parsed bucket", thread::current().id());

            match bucket.header.kind {
                MessageType::Terminate => {
                    // println!("#{:?} - terminate message.", thread::current().id());
                    sync.user.signal();
                    break
                },
                MessageType::Intercept => {
                    // println!("#{:?} - redirecting interception", thread::current().id());
                    let interception = unsafe { Interception::from_raw(buffer.as_mut_ptr()
                                    .offset(mem::size_of::<Syncronizers>() as isize)) };

                    let map = callbacks.read().expect("Unable to unlock callbacks for reading");

                    if let Some(callback) = map.get(&interception.guard_id) {
                        bucket.set_action(&mut buffer, callback(interception));
                    } else {
                        bucket.set_action(&mut buffer, default(interception));
                    }

                },
                _ => {}
            }

            // println!("#{:?} message-ready, notifying back.", thread::current().id());

            // TODO: Convert this check into something more fancy.
            if (bucket.header.control & ControlFlags::SE_MESSAGE_ASYNCHRONOUS) != ControlFlags::SE_MESSAGE_ASYNCHRONOUS {
                sync.user.signal();
            }
        }

        // just a (leak) hack to avoid unstable free
        mem::forget(buffer);
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