extern crate indicatif;

use std::sync::mpsc::{Receiver, Sender};

use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::{thread, time};

#[derive(Clone, Copy)]
pub enum MessageType {
    Progress,
    CreateProgress,
    CloseProgress,
    IncProgress,
    Spinner,
    Exit,
    Close,
}

pub struct ShellMessage {
    kind: MessageType,
    content: String,
    id: u32,
    progress: usize,
}

impl ShellMessage {
    pub fn new(tx: &Sender<ShellMessage>, content: String, id: u32, total: usize) -> ShellMessage {
        ShellMessage::update_bar(
            &tx,
            ShellMessage {
                content: content.clone(),
                kind: MessageType::CreateProgress,
                id: id,
                progress: total,
            },
        );
        ShellMessage {
            content: content,
            kind: MessageType::Progress,
            id: id,
            progress: total,
        }
    }

    pub fn kind(&self) -> MessageType {
        self.kind
    }
    pub fn inc(&self, tx: &Sender<ShellMessage>, amount: usize) -> () {
        ShellMessage::update_bar(
            &tx,
            ShellMessage {
                content: "".to_string(),
                kind: MessageType::IncProgress,
                id: self.id,
                progress: amount,
            },
        );
    }

    pub fn set_progress(&self, tx: &Sender<ShellMessage>, completed_progress: usize) -> () {
        ShellMessage::update_bar(
            &tx,
            ShellMessage {
                content: "".to_string(),
                kind: MessageType::Progress,
                id: self.id,
                progress: completed_progress,
            },
        );
    }
    pub fn complete(&self, tx: &Sender<ShellMessage>) -> () {
        ShellMessage::update_bar(
            &tx,
            ShellMessage {
                content: "".to_string(),
                kind: MessageType::CloseProgress,
                id: self.id,
                progress: self.progress,
            },
        );
    }

    pub fn sleep_bar(tx: &Sender<ShellMessage>, sleep_seconds: usize) -> () {
        let bar = ShellMessage::new(
            tx,
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}".to_string(),
            0,
            sleep_seconds,
        );
        for i in 1..sleep_seconds {
            bar.set_progress(tx, i);
            thread::sleep(time::Duration::from_secs(1));
        }
        bar.complete(tx);
    }

    pub fn send(
        messenger: &Sender<ShellMessage>,
        content: String,
        kind: MessageType,
        id: u32,
    ) -> bool {
        //TODO:REVIEW:BUG: If we try to write a bigg spinner, will ignore chars from i think 256 as limit
        ShellMessage::update_bar(
            &messenger,
            ShellMessage {
                content: content,
                kind: kind,
                id: id,
                progress: 0,
            },
        );
        true
    }

    fn update_bar(tx: &Sender<ShellMessage>, message: ShellMessage) -> () {
        tx.send(message).unwrap();
    }

    pub fn exit(tx: &Sender<ShellMessage>, content: String) -> () {
        ShellMessage::update_bar(
            &tx,
            ShellMessage {
                content: content,
                kind: MessageType::Exit,
                id: 0,
                progress: 0,
            },
        );
    }
}

#[allow(unused_variables)]
pub fn create_messenger(
    rx: Receiver<ShellMessage>,
    elapse: Option<time::Duration>,
    rows: usize,
) -> thread::JoinHandle<()> {
    let mut container: HashMap<usize, ProgressBar> = HashMap::new();
    let mut progresses: HashMap<usize, ProgressBar> = HashMap::new();

    let tt = thread::spawn(move || loop {
        let message = rx.recv().unwrap();

        let message_id: usize = message.id as usize;

        let elapse = elapse.unwrap_or(time::Duration::from_millis(10));

        match message.kind() {
            MessageType::Exit => {
                if container.contains_key(&message_id) {
                    container[&message_id].finish_with_message(message.content);
                };
                break;
            }
            MessageType::Close => {
                if container.contains_key(&message_id) {
                    container[&message_id].finish_with_message(message.content);
                    container.remove(&message_id);
                } else {
                    let bar = ProgressBar::new_spinner();
                    bar.set_style(
                        ProgressStyle::default_spinner()
                            .tick_chars(".·: ")
                            .template("{spinner:.dim.bold} {wide_msg}")
                            .expect("Error"),
                    );
                    bar.finish_with_message(message.content);
                }

                thread::sleep(elapse);
            }
            MessageType::CreateProgress => {
                progresses.insert(message_id, ProgressBar::new(message.progress as u64));
                progresses[&message_id].set_style(
                    ProgressStyle::default_bar()
                        .template(&message.content)
                        .expect("Error")
                        .progress_chars("##-"),
                );
            }
            MessageType::CloseProgress => {
                progresses[&message_id].finish();
                progresses.remove(&message_id);
            }
            MessageType::Progress => {
                progresses[&message_id].set_position(message.progress as u64);
            }
            MessageType::IncProgress => {
                progresses[&message_id].inc(message.progress as u64);
            }
            MessageType::Spinner => {
                if !container.contains_key(&message_id) {
                    container.insert(message_id, ProgressBar::new_spinner());
                    container[&message_id].set_style(
                        ProgressStyle::default_spinner()
                            .tick_chars(".·: ")
                            .template("{spinner:.dim.bold} {wide_msg}")
                            .expect("Error"),
                    );
                }
                container[&message_id].set_message(message.content);
                thread::sleep(elapse);
            }
        }
    });

    tt
}
