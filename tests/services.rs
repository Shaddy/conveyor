extern crate create_service;

use create_service::WindowsService;

#[test]
fn test_windows_service_object() {
    let mut service = WindowsService::new("SampleService", "");
    assert!(service.open().is_ok())
}

#[test]
fn test_query_service() {

    let mut service = WindowsService::new("SampleService", "");
    let handle = service.open().expect("Can't open the service");
    service.query(handle);
}