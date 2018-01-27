extern crate indicatif;

use std::sync::mpsc::{channel, Receiver, Sender};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::{thread, time};
use std::iter::Iterator;
use std::collections::hash_map::Entry::{Occupied, Vacant};

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
    id: u32,
}

impl ShellMessage {
    pub fn new(content: String, out_type: MessageType, id: u32) -> ShellMessage {
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
        id: u32,
    ) -> bool {
        tx.send(ShellMessage {
            content: content,
            out_type: out_type,
            id: id,
        }).unwrap();
        true
    }
}

pub fn thread_printer(rx: Receiver<ShellMessage>, limit: usize) -> (thread::JoinHandle<()>, MultiProgress) {
    let m = MultiProgress::new();
    let mut container: Vec<ProgressBar> = Vec::new();

    (0..limit).for_each(|idx| {
        container.push(m.add(ProgressBar::new_spinner()));
    });

    let tt = thread::spawn(move || {
        loop {
            let message: ShellMessage = rx.recv().unwrap();

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


            match message.out_type {
                MessageType::exit => {
                    // let sp = &container[0];
                    container[0].finish_with_message(&message.content);
                    break;
                }
                MessageType::close => {
                    // let sp = container.to_vec()[0];
                    container[0].finish_with_message(&message.content);
                    container.remove(0);
                }
                MessageType::progress => {
                    // let sp = &container[0];
                    container[0].set_message(&message.content);
                }
                MessageType::spinner => {
                    // let sp = &container[0];
                    container[0].set_message(&message.content);
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
