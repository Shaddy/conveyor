#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]
extern crate conveyor;
extern crate winapi;

use winapi::{FILE_READ_ACCESS, FILE_WRITE_ACCESS, METHOD_BUFFERED};
use conveyor::iochannel::Device;
use conveyor::iochannel::IoCtl;


describe! ioctl_create_partition {
    before_each {
        let ioctl = IoCtl::from(0xB080E800);
    }

    it "device_type is SENTRY_DEVICE_TYPE" {
        assert_eq!(ioctl.device_type, 0xB080);
    }

    it "function is PARTITION_CREATE" {
        assert_eq!(ioctl.function, 0x0A00);
    }

    it "method is METHOD_BUFFERED" {
        assert_eq!(ioctl.method, METHOD_BUFFERED);
    }

    it "access is FILE_READ_ACCESS | FILE_WRITE_ACCESS" {
        assert_eq!(ioctl.access, FILE_READ_ACCESS | FILE_WRITE_ACCESS);
    }
}

#[test]
#[ignore]
fn test_device_is_able_to_open() {
    assert!( Device::open("any_kind_of_file").is_ok() );
}