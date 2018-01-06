// use std::fmt;
// use std::sync::Arc;

use super::{memory, misc};

use super::iochannel::{Device};
use std::str;

// use super::symbols::parser::Error;
const MAX_SEARCH_SIZE: usize = 0x10000;

pub fn pattern(device: &Device, name: &str, pattern: &[u8], neighbour: &str) -> Option<usize> {
    if let Some(driver) = misc::Drivers::contains(name) {

        // TODO: Create an IOCTL to retrieve the procedure address
        let address = misc::get_proc_addr(driver.base(), neighbour)
                                .expect(&format!("{}", neighbour));


        let map = memory::Map::new(device, driver.base() + address, MAX_SEARCH_SIZE);

        //
        // this code looks with side-effects but its verified, there is an algorithm from str
        // that checks subsets, gaining close to 30% of performance
        //
        // https://play.rust-lang.org/?gist=a645b02c3fe5770805dd531b41eecb32&version=nightly
        //
        let code    = unsafe { str::from_utf8_unchecked(map.as_slice())};
        let pattern = unsafe { str::from_utf8_unchecked(pattern) } ;
        
        if code.contains(pattern) {
            return code.find(pattern)
        }
    } 

    None
}