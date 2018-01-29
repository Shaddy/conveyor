// Copyright © ByteHeed.  All rights reserved.

use std;
use super::core::WindowsService;
use super::structs::ServiceStatus;

use std::sync::mpsc::Sender;
use super::cli::output::{MessageType, ShellMessage};


fn full_driver_path(name: &str) -> String {
    let mut path = std::env::current_dir().expect("error getting current dir");
    path.push(format!("{}.sys", name));
    path.to_str()
        .expect("Failed to convert to string")
        .to_string()
}

pub fn query(name: &str, messenger: &Sender<ShellMessage>) {
    ShellMessage::send(messenger, format!("Querying {}", name), MessageType::Spinner, 0);

    ShellMessage::send(
        messenger,
        format!(
            "{:?}",
            WindowsService::new(name, &full_driver_path(name)).query()
        ),
        MessageType::Close,
        1,
    );
}

pub fn stop(name: &str, messenger: &Sender<ShellMessage>) {
    ShellMessage::send(messenger, format!("Service {} Stopping ", name), MessageType::Spinner, 0);
    WindowsService::new(name, &full_driver_path(name)).stop();
    ShellMessage::send(messenger, format!("Service {} stopped", name), MessageType::Close, 0);
}

pub fn start(name: &str, messenger: &Sender<ShellMessage>) {
    ShellMessage::send(messenger, format!("Service {} starting", name), MessageType::Spinner, 0);
    WindowsService::new(name, &full_driver_path(name)).start();
    ShellMessage::send(messenger, format!("Service {} started", name), MessageType::Close, 0);
}

pub fn install(name: &str, messenger: &Sender<ShellMessage>) {
    ShellMessage::send(messenger, format!("Service {} installing", name), MessageType::Spinner, 0);
    WindowsService::new(name, &full_driver_path(name)).install();
    ShellMessage::send(
        messenger,
        format!("Service {} has been successfully installed", name),
        MessageType::Close,
        0,
    );
}

pub fn remove(name: &str, messenger: &Sender<ShellMessage>) {
    ShellMessage::send(messenger, format!("Service {} removing", name), MessageType::Spinner, 0);

    WindowsService::new(name, &full_driver_path(name)).remove();
    ShellMessage::send(
        messenger,
        format!("Service {} has been successfully removed", name),
        MessageType::Close,
        0,
    );

}

pub fn update(name: &str, messenger: &Sender<ShellMessage>) {
    ShellMessage::send(messenger, format!("Service {} updating", name), MessageType::Spinner, 0);
    // debug!(logger, "updating {}", name);

    let mut service = WindowsService::new(name, &full_driver_path(name));

    if service.exists() {
        service.remove();
        ShellMessage::send(
            messenger,
            format!("Service {} has been successfully removed", name),
            MessageType::Spinner,
            0,
        );
    }
    service.install();
    ShellMessage::send(
        messenger,
        format!("Service {} has been successfully updated", name),
        MessageType::Close,
        0,
    );

}

pub fn reinstall(name: &str, messenger: &Sender<ShellMessage>) {
    ShellMessage::send(
        messenger,
        format!("Service {} reinstalling ", name),
        MessageType::Spinner,
        0,
    );

    let mut service = WindowsService::new(name, &full_driver_path(name));

    if service.exists() {
        ShellMessage::send(
            messenger,
            format!("Service {} found, stopping", name),
            MessageType::Spinner,
            0,
        );
        service.stop();

        let mut timeout = std::time::Duration::from_secs(60);
        let wait = std::time::Duration::from_secs(1);

        while let ServiceStatus::PausePending = service.query().status {
            ShellMessage::send(
                messenger,
                format!(
                    "Service {} stop is pending, waiting {} seconds.",
                    service.name(),
                    timeout.as_secs()
                ),
                MessageType::Spinner,
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
            messenger,
            format!("Service {} removed succesfully", name),
            MessageType::Spinner,
            0,
        );
    }
    ShellMessage::send(messenger, format!("Installing {} service...", name), MessageType::Spinner, 1);

    service.install();

    ShellMessage::send(messenger, format!("Service {} succesfully reinstalled!",name), MessageType::Close, 1);
}
