
extern crate indicatif;

use std::sync::mpsc::{Receiver, Sender};

use std::collections::HashMap;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{thread, time};
use std::iter::Iterator;

#[derive(Clone, Copy)]
pub enum MessageType {
    Progress,
    CreateProgress,
    CloseProgress,
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
    pub fn new(tx: &Sender<ShellMessage>,content: String, id: u32, total: usize) -> ShellMessage {

        ShellMessage::update_bar(&tx, ShellMessage {
            content: content.clone(),
            kind: MessageType::CreateProgress,
            id: id,
            progress: total,
        });
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

    pub fn set_progress(&self, tx: &Sender<ShellMessage>, completed_progress:usize) -> (){
        ShellMessage::update_bar(&tx, ShellMessage{
            content: "".to_string(),
            kind: MessageType::Progress,
            id: self.id,
            progress: completed_progress,
        });
    }
    pub fn complete(&self, tx: &Sender<ShellMessage>) -> (){
        ShellMessage::update_bar(&tx, ShellMessage{
            content: "".to_string(),
            kind: MessageType::CloseProgress,
            id: self.id,
            progress: self.progress,
        });
    }


    pub fn send(
        messenger: &Sender<ShellMessage>,
        content: String,
        kind: MessageType,
        id: u32,
    ) -> bool {
        ShellMessage::update_bar(
            &messenger,
            ShellMessage {
                content: content,
                kind: kind,
                id: id,
                progress: 0
            },
        );
        true
    }

    fn update_bar(tx: &Sender<ShellMessage>, message: ShellMessage) -> () {
        tx.send(message).unwrap();
    }

    pub fn Exit(tx: &Sender<ShellMessage>, content: String) -> () {
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

pub fn thread_printer(
    rx: Receiver<ShellMessage>,
    rows: usize,
) -> (thread::JoinHandle<()>, MultiProgress) {
    let multi_progress = MultiProgress::new();
    let mut container: Vec<ProgressBar> = Vec::new();
    let mut progresses: Vec<ProgressBar> = Vec::new();
    let mut totals: HashMap<usize, usize> = HashMap::new();


    (0..rows).for_each(|_| {
        container.push(multi_progress.add(ProgressBar::new_spinner()));
    });
    (0..rows).for_each(|_| {
        progresses.push(multi_progress.add(ProgressBar::new(100)));
    });



    let tt = thread::spawn(move || {
        loop {
            let message = rx.recv().unwrap();

            /*
            obtenemos nuestro indice desde el manager

            Si nuestro indice es == -1 en status

            Buscamos el proximo indice que este a 0 en status y lo asignamos

            Lo asignamos a manager y cambiamos el estado a 1

            objetos a tocar: manager, satus, message.id,

            finalmente asignamos el nuevo contenedor a sp

            let sp = container.get(manager.get(&message.id).unwrap()).unwrap();
            y borramos las asignaciones manuales de let sp, dentro del match
            */

            let message_id: usize = message.id as usize;

            // let message_id = *message.id;
            match message.kind() {
                MessageType::Exit => {
                    // let sp = &container[0];
                    container[message_id].finish_with_message(&message.content);
                    break;
                }
                MessageType::Close => {
                    // let sp = container.to_vec()[0];
                    container[message_id].finish_with_message(&message.content);
                    container.remove(message_id);
                }
                MessageType::CreateProgress => {
                    progresses[message_id].set_style(ProgressStyle::default_bar()
                                                    .template(&message.content)
                                        .progress_chars("##-"));
                    progresses[message_id].set_length(message.progress as u64);
                    totals.insert(message_id, message.progress);
                }
                MessageType::CloseProgress => {
                    progresses[message_id].finish();

                }
                MessageType::Progress => {
                    // let sp = &container[0];
                    // 100, totals[message_id], message.progress
                    // let advance = (message.progress/totals[&message_id]) * 100;
                    progresses[message_id].set_position(message.progress as u64);
                }
                MessageType::Spinner => {
                    // let sp = &container[0];
                    container[message_id].set_message(&message.content);
                }
                _ => (),
            }
            // thread::sleep(time::Duration::from_secs(2))
            // m.join_and_clear().unwrap();
            thread::sleep(time::Duration::from_millis(500));
        }

        // multi_progress.join();
        // helper_stdout.finish_with_message("All bars closed!");
    });

    (tt, multi_progress)
}
