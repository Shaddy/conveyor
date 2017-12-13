extern crate byteorder;
extern crate winapi;
extern crate kernel32;


use super::sync::Event;
use std::mem;
use std::thread;

use std::fmt::Debug;
use std::fmt;
    

const NUM_OF_THREADS: usize = 4;
const BUCKET_SIZE: usize = (240 + 16);


#[repr(C)]
pub struct Bucket {
    pub user: Event,
    pub kernel: Event,
    data: [u8; 240]
}

impl Debug for Bucket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bucket({:?}, {:?}, {:?})", self.user, self.kernel, 
                unsafe { MessageHeader::from_raw(self.data.as_ptr()) })
    }
}

#[derive(Debug)]
enum MessageType {
    Intercept = 0x0000000000000000,
    Error,
    Terminate
}

#[derive(Debug)]
#[repr(C)]
struct MessageHeader {
    id: u64,
    kind: MessageType
}

impl MessageHeader {
    pub unsafe fn from_raw(ptr: *const u8) -> MessageHeader {
        mem::transmute_copy(&*ptr)
    }
}

#[derive(Debug)]
#[repr(C)]
struct Interception {
    header: MessageHeader,
    region_id: u64,
    processor: u8,
    process: u64,
    address: u64,
    access: u32,
    flags: u16,
    context: u64,
    action: u64
}

impl Interception {
    pub unsafe fn from_raw(ptr: *const u8) -> Interception {
        mem::transmute_copy(&*ptr)
    }
}

enum Message {
    Intercept(Interception),
    Terminate
}

impl Bucket {
    pub fn slice_buckets(ptr: u64, capacity: usize) -> Vec<Vec<u8>> {
        let chunks = BUCKET_SIZE;

        let ptr: *mut u8 = ptr as *mut u8;

        let size = capacity / chunks;

        let buffers = unsafe {
            let mut buffers: Vec<Vec<u8>> = Vec::new();
            
            for current in (0..capacity).step_by(BUCKET_SIZE) {
                buffers.push(Vec::from_raw_parts(ptr.offset(current as isize), size, size));
            };
            
            buffers
        };

        buffers
    }

    fn message(&self) -> Message {
        match unsafe { MessageHeader::from_raw(self.data.as_ptr()) }.kind {
            MessageType::Intercept => Message::Intercept(unsafe { 
                Interception::from_raw(self.data.as_ptr())
                        // // skip message header
                        // .offset(mem::size_of::<MessageHeader>() as isize)) 
            }),
            MessageType::Terminate => Message::Terminate,
            MessageType::Error => panic!("Error message, aborting!"),
        }
    }

    // TODO: add generic callback as parameter
    pub fn handler(mut buffer: Vec<u8>) {
        println!("running-thread #{:?} (waiting)", thread::current().id());
        let bucket = unsafe{ Bucket::from_raw(buffer.as_mut_ptr()) } ;
        loop {
            println!("#{:?} - waiting for new messsage.", thread::current().id());
            bucket.kernel.wait();

            println!("received-bucket: {:?}", bucket);


            match bucket.message() {
                Message::Terminate => break,
                Message::Intercept(_interception) => {
                    // TODO: write interception to callback code
                    // 
                    // bucket.set_action(callback(interception));
                },
            }

            println!("#{:?} message-ready, notifying back.", thread::current().id());
            bucket.user.signal();
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
