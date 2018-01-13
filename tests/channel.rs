#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]
extern crate conveyor;
extern crate winapi;

use winapi::um::winioctl;

use conveyor::iochannel::Device;
use conveyor::iochannel::IoCtl;


describe! ioctl_from_create_partition {
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
        assert_eq!(ioctl.method, winioctl::METHOD_BUFFERED);
    }

    it "access is FILE_READ_ACCESS | FILE_WRITE_ACCESS" {
        assert_eq!(ioctl.access, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);
    }
}

describe! ioctl_into_create_partition {
    before_each {
        let ioctl = IoCtl::new(0xB080, 0x0A00, None, None);
    }

    it "io code is 0xB080E800" {
        assert_eq!(ioctl.code(), 0xB080E800);
    }
}

describe! ioctl_into_free_memory {
    before_each {
        let ioctl = IoCtl::new(0xB080, 0x0A51, None, None);
    }

    it "from io code is 0xb080e944" {
        assert_eq!(IoCtl::from(0xb080e944).code(), ioctl.code());
    }

    it "from io code function is 0x0A51" {
        assert_eq!(IoCtl::from(0xb080e944).function, 0xA51);
    }

    it "io code is 0xb080e944" {
        assert_eq!(ioctl.code(), 0xb080e944);
    }
}

describe! ioctl_into_write_memory {
    before_each {
        let ioctl = IoCtl::new(0xB080, 0x0A59, None, None);
    }

    it "from io code is 0xb080e944" {
        assert_eq!(IoCtl::from(0xb080e964).code(), ioctl.code());
    }

    it "from io code function is 0x0A51" {
        assert_eq!(IoCtl::from(0xb080e964).function, 0xA59);
    }

    it "io code is 0xb080e944" {
        assert_eq!(ioctl.code(), 0xb080e964);
    }
}


#[test]
#[ignore]
fn test_device_is_able_to_open() {
    assert!( Device::open("any_kind_of_file").is_ok() );
}