extern crate winapi;
extern crate kernel32;

use std::io::Error;
use std::ptr::{null_mut, null};
use std::ops::Deref;

use self::kernel32::{ WaitForSingleObject, 
                SetEvent,
                ResetEvent,
                //CloseHandle,
                //OpenEventW,
                CreateEventW};


#[derive(Debug)]
pub struct Event(winapi::HANDLE);

impl Event {

    pub fn _create() -> winapi::HANDLE {
        let (manual, init) = (false, false);

        unsafe {
            CreateEventW(null_mut(),
                        manual as winapi::BOOL,
                        init as winapi::BOOL,
                        null())
        }
    }

    pub fn _new() -> Event {
        Event(Event::_create())
    }

    pub fn _as_u64(&self) -> u64 {
        self.0 as u64
    }

    pub fn _reset(&self) -> &Self {
        if unsafe { ResetEvent(self.0) } == 0 {
            panic!("Failed to wait for the event: {}", 
                    Error::last_os_error());
        }

        self
    }

    pub fn signal(&self) -> &Self {

        if unsafe { SetEvent(self.0) } == 0 {
            panic!("Failed to signal event: {}", 
                    Error::last_os_error());
        }

        self
    }

    pub fn wait(&self) {
        let rc = unsafe { WaitForSingleObject(self.0, winapi::INFINITE) };
        if rc == winapi::WAIT_FAILED {
            panic!("Failed to wait for the event: {}", 
                    Error::last_os_error());
        }
    }

}

impl Into<u64> for Event {
    fn into(self) -> u64 {
        self.0 as u64
    }
}

impl From<u64> for Event {
    fn from(handle: u64) -> Self {
        Event(handle as winapi::HANDLE)
    }
}

impl From<winapi::HANDLE> for Event {
    fn from(handle: winapi::HANDLE) -> Self {
        Event(handle)
    }
}

//
// dropping is disabled due to ownership of event
//
// impl Drop for Event {
//     fn drop(&mut self) {
//         println!("droping event {:?}", self.0);

//         unsafe {
//             CloseHandle(self.0);
//         }
//     }
// }

impl Deref for Event {
    type Target = winapi::HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// impl Deref for Event {

//     fn deref(&self) -> winapi::HANDLE {
//         self.0
//     }
// }
