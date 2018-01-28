extern crate indicatif;

use std::sync::mpsc::{Receiver, Sender};

use indicatif::{MultiProgress, ProgressBar};
use std::{thread, time};
use std::iter::Iterator;

#[derive(Clone, Copy)]
pub enum MessageType {
    Progress,
    Spinner,
    Exit,
    Close,
}

pub struct ShellMessage {
    kind: MessageType,
    content: String,
    _id: u32,
}

impl ShellMessage {
    pub fn new(content: String, kind: MessageType, id: u32) -> ShellMessage {
        ShellMessage {
            content: content,
            kind: kind,
            _id: id,
        }
    }

    pub fn kind(&self) -> MessageType {
        self.kind
    }

    pub fn send(
        messenger: &Sender<ShellMessage>,
        content: String,
        kind: MessageType,
        id: u32,
    ) -> bool {
        messenger.send(ShellMessage {
            content: content,
            kind: kind,
            _id: id,
        }).unwrap();
        true
    }
}

pub fn thread_printer(rx: Receiver<ShellMessage>, rows: usize) -> (thread::JoinHandle<()>, MultiProgress) {
    let multi_progress = MultiProgress::new();
    let mut container: Vec<ProgressBar> = Vec::new();

    (0..rows).for_each(|_| { container.push(multi_progress.add(ProgressBar::new_spinner())); });

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


            match message.kind() {
                MessageType::Exit => {
                    // let sp = &container[0];
                    container[0].finish_with_message(&message.content);
                    break;
                }
                MessageType::Close => {
                    // let sp = container.to_vec()[0];
                    container[0].finish_with_message(&message.content);
                    container.remove(0);
                }
                MessageType::Progress => {
                    // let sp = &container[0];
                    container[0].set_message(&message.content);
                }
                MessageType::Spinner => {
                    // let sp = &container[0];
                    container[0].set_message(&message.content);
                }
            }
            // thread::sleep(time::Duration::from_secs(2))
            // m.join_and_clear().unwrap();
            thread::sleep(time::Duration::from_millis(500));
        }

        // helper_stdout.finish_with_message("All bars closed!");
    });

    (tt, multi_progress)
}
