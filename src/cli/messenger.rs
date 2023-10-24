// use std::sync::{Mutex, Sender};
// use lazy_static::lazy_static;
//
// lazy_static! {
//     static ref GLOBAL_MESSENGER: Mutex<Sender<ShellMessage>> = {
//         let (sender, receiver) = channel();
//         let messenger = create_messenger(receiver, None, 20);
//         Mutex::new(messenger)
//     };
// }
//
// // Macro para enviar mensajes
// macro_rules! send_msg {
//     ($msg:expr) => {
//         {
//             let messenger = GLOBAL_MESSENGER.lock().unwrap();
//             // ShellMessage::send(&messenger, $msg).unwrap();
//         }
//     };
// }
//
//
// lazy_static! {
//     static ref GLOBAL_MESSENGER: Mutex<Sender<YourMessageType>> = {
//         Mutex::new(create_your_messenger());
//     }
// }
//
