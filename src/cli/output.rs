extern crate indicatif;

use std::sync::mpsc::{channel, Receiver, Sender};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::{thread, time};

#[derive(Clone)]
pub enum MessageType {
    progress,
    spinner,
    exit,
    close,
}

pub struct ShellMessage {
    out_type: MessageType,
    content: String,
    id: usize,
}

impl ShellMessage {
    pub fn new(content: String, out_type: MessageType, id: usize) -> ShellMessage {
        ShellMessage {
            content: content,
            out_type: out_type,
            id: id,
        }
    }

    pub fn send(
        tx: &Sender<ShellMessage>,
        content: String,
        out_type: MessageType,
        id: usize,
    ) -> bool {
        tx.send(ShellMessage {
            content: content,
            out_type: out_type,
            id: id,
        }).unwrap();
        true
    }
}

pub fn thread_printer(rx: Receiver<ShellMessage>) -> (thread::JoinHandle<()>, MultiProgress) {
    let m = MultiProgress::new();
    let mut container: HashMap<usize, ProgressBar> = HashMap::new();
    (0..5).for_each(|idx| {
        container.insert(idx as usize, m.add(ProgressBar::new_spinner()));
    });
    let tt = thread::spawn(move || {
        loop {
            let message: ShellMessage = rx.recv().unwrap();
            match message.out_type {
                MessageType::exit => {
                    let sp = container.get(&message.id).unwrap();
                    sp.finish_with_message(&message.content);
                    break;
                }
                MessageType::close => {
                    let sp = container.get(&message.id).unwrap();
                    sp.finish_with_message(&message.content);
                }
                MessageType::progress => {
                    let sp = container.get(&message.id).unwrap();
                    sp.set_message(&message.content);
                }
                MessageType::spinner => {
                    let sp = container.get(&message.id).unwrap();
                    sp.set_message(&message.content);
                }
                _ => println!("Unknown message"),
            }
            // thread::sleep(time::Duration::from_secs(2))
            // m.join_and_clear().unwrap();
            thread::sleep(time::Duration::from_millis(500));
        }

        // helper_stdout.finish_with_message("All bars closed!");
    });
    (tt, m)
}
