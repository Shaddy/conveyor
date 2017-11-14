extern crate conveyor;

use conveyor::{WindowsService, ServiceError};
use conveyor::structs::{ServiceStatus};

// #[test]
// fn test_unprivileged_user_cant_query_services() {
//     let invalid_service = WindowsService::new("", "").open();
//     assert!(invalid_service.is_err());
//     assert!(invalid_service.unwrap_err() == ServiceError::AccessViolation)
// }

#[test]
fn test_service_does_not_exist() {
    let service = WindowsService::new("ServiceThatDoesNotExist", "").open();
    assert!(service.is_err());
    assert!(service.unwrap_err() == ServiceError::ServiceDoesNotExist);
}

#[test]
fn test_query_service() {

    let service = WindowsService::new("LxssManager", "");
    let handle = service.open().expect("Can't open the service");
    let info = service.query(handle);

    assert!(info.status == ServiceStatus::Running);
    assert!(info.kind.bits() == 0x30)
}

#[test]
fn test_service_exists() {
    assert!(WindowsService::new("LxssManager", "").exists() == true);
    assert!(WindowsService::new("ServiceThatNotExists", "").exists() == false);
}