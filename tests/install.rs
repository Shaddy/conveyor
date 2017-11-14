extern crate create_service;

use create_service::WindowsService;

#[test]
fn test_windows_service_object() {
    let mut service = WindowsService::new("service-name", "path/to/executable");
    service.install();
}