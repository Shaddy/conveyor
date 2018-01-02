extern crate winapi;

use std::io::Error;
use std::ptr::{null_mut, null};
use std::ops::Deref;

use self::winapi::um::synchapi;

use self::winapi::um::winbase;
use self::winapi::um::winnt;
use self::winapi::shared::minwindef;


#[derive(Debug)]
pub struct Event(winnt::HANDLE);

impl Event {

    pub fn _create() -> winnt::HANDLE {
        let (manual, init) = (false, false);

        unsafe {
            synchapi::CreateEventW(null_mut(),
                        manual as minwindef::BOOL,
                        init as minwindef::BOOL,
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
        if unsafe { synchapi::ResetEvent(self.0) } == 0 {
            panic!("Failed to wait for the event: {}", 
                    Error::last_os_error());
        }

        self
    }

    pub fn signal(&self) -> &Self {

        if unsafe { synchapi::SetEvent(self.0) } == 0 {
            panic!("Failed to signal event: {}", 
                    Error::last_os_error());
        }

        self
    }

    pub fn wait(&self) {
        let rc = unsafe { synchapi::WaitForSingleObject(self.0, winbase::INFINITE) };
        if rc == winbase::WAIT_FAILED {
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
        Event(handle as winnt::HANDLE)
    }
}

impl From<winnt::HANDLE> for Event {
    fn from(handle: winnt::HANDLE) -> Self {
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
    type Target = winnt::HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// impl Deref for Event {

//     fn deref(&self) -> HANDLE {
//         self.0
//     }
// }
