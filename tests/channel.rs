#[test]
fn test_io_call() {
    let device = Device::open("device_name").expect("Can't open device");
    let result = device.call(IO_CODE);
}