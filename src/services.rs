extern crate conveyor;
extern crate slog;

use std;
use self::conveyor::WindowsService;

use slog::*;

pub fn query_service(name: &str, logger: &Logger) {
    debug!(logger, "querying {:?}", name);

    let mut path = std::env::current_dir().expect("error getting current dir");
    path.push(format!("{}.sys", name));

    println!("{:?}", WindowsService::new(name, path.to_str().unwrap()).query());
}

pub fn stop_service(name: &str, logger: &Logger) {
    debug!(logger, "stopping {:?}", name);

    let mut path = std::env::current_dir().expect("error getting current dir");
    path.push(format!("{}.sys", name));

    WindowsService::new(name, path.to_str().unwrap()).stop();
}

pub fn start_service(name: &str, logger: &Logger) {
    debug!(logger, "starting {:?}", name);

    let mut path = std::env::current_dir().expect("error getting current dir");
    path.push(format!("{}.sys", name));

    WindowsService::new(name, path.to_str().unwrap()).start();
}

pub fn install_service(name: &str, logger: &Logger) {
    debug!(logger, "installing {:?}", name);

    let mut path = std::env::current_dir().expect("error getting current dir");
    path.push(format!("{}.sys", name));

    WindowsService::new(name, path.to_str().unwrap()).install();
}

pub fn remove_service(name: &str, logger: &Logger) {
    debug!(logger, "removing {:?}", name);

    let mut path = std::env::current_dir().expect("error getting current dir");
    path.push(format!("{}.sys", name));

    WindowsService::new(name, path.to_str().unwrap()).remove();
}

pub fn update_service(name: &str, logger: &Logger) {
    debug!(logger, "updating {}", name);

    let mut path = std::env::current_dir().expect("error getting current dir");
    path.push(format!("{}.sys", name));

    WindowsService::new(name, path.to_str().unwrap()).remove();
    WindowsService::new(name, path.to_str().unwrap()).install();
}

pub fn run_service(name: &str, logger: &Logger) {
    debug!(logger, "udpating & starting => {:?}", name);

    let mut path = std::env::current_dir().expect("error getting current dir");
    path.push(format!("{}.sys", name));

    let service = WindowsService::new(name, path.to_str().unwrap());

    if service.exists() {
        service.stop().remove().install().start();
    } else {
        service.install().start();
    }
}