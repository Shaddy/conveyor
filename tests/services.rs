extern crate conveyor;

use conveyor::WindowsService;

#[test]
fn test_windows_service_object() {
    let service = WindowsService::new("AdobeUpdateService", "");
    assert!(service.open().is_ok())
}

#[test]
fn test_query_service() {

    let service = WindowsService::new("AdobeUpdateService", "");
    let handle = service.open().expect("Can't open the service");
    let info = service.query(handle);

    println!("{:?}", info);
    assert!(1 == 0)
}