// Copyright © ByteHeed.  All rights reserved.

use std;
use super::core::WindowsService;
use super::structs::ServiceStatus;

use std::sync::mpsc::Sender;
use super::cli::output::{MessageType, ShellMessage};

use super::slog::*;

fn full_driver_path(name: &str) -> String {
    let mut path = std::env::current_dir().expect("error getting current dir");
    path.push(format!("{}.sys", name));
    path.to_str()
        .expect("Failed to convert to string")
        .to_string()
}

pub fn query(name: &str, tx: &Sender<ShellMessage>) {
    ShellMessage::send(&tx, format!("Querying {:?}", name), MessageType::spinner, 0);

    ShellMessage::send(
        &tx,
        format!(
            "{:?}",
            WindowsService::new(name, &full_driver_path(name)).query()
        ),
        MessageType::close,
        1,
    );
}

pub fn stop(name: &str, tx: &Sender<ShellMessage>) {
    ShellMessage::send(&tx, format!("stopping {:?}", name), MessageType::spinner, 0);
    WindowsService::new(name, &full_driver_path(name)).stop();
    ShellMessage::send(&tx, format!("stopped {:?}", name), MessageType::close, 0);
}

pub fn start(name: &str, tx: &Sender<ShellMessage>) {
    ShellMessage::send(&tx, format!("starting {:?}", name), MessageType::spinner, 0);
    WindowsService::new(name, &full_driver_path(name)).start();
    ShellMessage::send(&tx, format!("started {:?}", name), MessageType::close, 0);
}

pub fn install(name: &str, tx: &Sender<ShellMessage>) {
    ShellMessage::send(&tx, format!("installing {:?}", name), MessageType::spinner, 0);
    WindowsService::new(name, &full_driver_path(name)).install();
    ShellMessage::send(
        &tx,
        format!("Service {:?} has been successfully installed", name),
        MessageType::close,
        0,
    );
}

pub fn remove(name: &str, tx: &Sender<ShellMessage>) {
    ShellMessage::send(&tx, format!("removing {:?}", name), MessageType::spinner, 0);

    WindowsService::new(name, &full_driver_path(name)).remove();
    ShellMessage::send(
        &tx,
        format!("Service {:?} has been successfully removed", name),
        MessageType::close,
        0,
    );

}

pub fn update(name: &str, tx: &Sender<ShellMessage>) {
    ShellMessage::send(&tx, format!("updating {}", name), MessageType::spinner, 0);
    // debug!(logger, "updating {}", name);

    let mut service = WindowsService::new(name, &full_driver_path(name));

    if service.exists() {
        service.remove();
        ShellMessage::send(
            &tx,
            format!("Service {:?} has been successfully removed", name),
            MessageType::spinner,
            0,
        );
    }
    service.install();
    ShellMessage::send(
        &tx,
        format!("Service {:?} has been successfully installed", name),
        MessageType::close,
        0,
    );

}

pub fn reinstall(name: &str, tx: &Sender<ShellMessage>) {
    ShellMessage::send(
        &tx,
        format!("reinstalling => {:?}", name),
        MessageType::spinner,
        0,
    );

    let mut service = WindowsService::new(name, &full_driver_path(name));

    if service.exists() {
        ShellMessage::send(
            &tx,
            format!("{} service found, stopping", name),
            MessageType::spinner,
            0,
        );
        service.stop();

        let mut timeout = std::time::Duration::from_secs(60);
        let wait = std::time::Duration::from_secs(1);

        while let ServiceStatus::PausePending = service.query().status {
            ShellMessage::send(
                &tx,
                format!(
                    "{}: stop is pending, waiting {} seconds.",
                    service.name(),
                    timeout.as_secs()
                ),
                MessageType::spinner,
                0,
            );
            // format!("{}: stop is pending, waiting {} seconds.", service.name(), timeout.as_secs());

            let service_name = service.name();
            timeout = timeout
                .checked_sub(wait)
                .ok_or_else(move || {
                    panic!(
                        "{}: reached timeout while stop is pending, exiting.",
                        service_name
                    );
                })
                .unwrap();

            std::thread::sleep(wait);
        }

        service.remove();
        ShellMessage::send(
            &tx,
            format!("Service {} removed succesfully", name),
            MessageType::close,
            0,
        );
    }
    ShellMessage::send(&tx, "Installing new service...".to_string(), MessageType::spinner, 1);

    service.install();

    ShellMessage::send(&tx, "Succesfully reinstalled!".to_string(), MessageType::close, 1);
}
