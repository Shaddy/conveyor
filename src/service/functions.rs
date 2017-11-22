// Copyright © ByteHeed.  All rights reserved.

use std;
use super::core::WindowsService;
use super::structs::ServiceStatus;

use super::slog::*;

fn full_driver_path(name: &str) -> String {
    let mut path = std::env::current_dir().expect("error getting current dir");
    path.push(format!("{}.sys", name));
    path.to_str().expect("Failed to convert to string").to_string()
}

pub fn query(name: &str, logger: &Logger) {
    debug!(logger, "querying {:?}", name);

    println!("{:?}", WindowsService::new(name, &full_driver_path(name)).query());
}

pub fn stop(name: &str, logger: &Logger) {
    debug!(logger, "stopping {:?}", name);

    WindowsService::new(name, &full_driver_path(name)).stop();
}

pub fn start(name: &str, logger: &Logger) {
    debug!(logger, "starting {:?}", name);

    WindowsService::new(name, &full_driver_path(name)).start();
}

pub fn install(name: &str, logger: &Logger) {
    debug!(logger, "installing {:?}", name);

    WindowsService::new(name, &full_driver_path(name)).install();
}

pub fn remove(name: &str, logger: &Logger) {
    debug!(logger, "removing {:?}", name);

    WindowsService::new(name, &full_driver_path(name)).remove();
}

pub fn update(name: &str, logger: &Logger) {
    debug!(logger, "updating {}", name);

    let service = WindowsService::new(name, &full_driver_path(name));

    if service.exists() {
        service.remove().install();
    } else {
        service.install();
    }
}

pub fn run(name: &str, logger: &Logger) {
    debug!(logger, "udpating & starting => {:?}", name);

    let service = WindowsService::new(name, &full_driver_path(name));

    if service.exists() {
        service.stop();

        let mut timeout = std::time::Duration::from_secs(60);
        let wait = std::time::Duration::from_secs(1);

        while let ServiceStatus::StopPending = service.query().status {
            println!("{}: stop is pending, waiting 60 seconds.", service.name());


            let service_name = service.name();
            timeout = timeout.checked_sub(wait).ok_or_else(move|| {
                panic!("{}: reached timeout while stop is pending, exiting.", service_name);
            }).unwrap();

            std::thread::sleep(wait);
        }

        service.remove().install().start();

    } else {
        service.install().start();
    }
}