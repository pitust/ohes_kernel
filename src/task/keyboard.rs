use crate::{print, println};
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
pub static KEY_QUEUE: OnceCell<ArrayQueue<pc_keyboard::DecodedKey>> = OnceCell::uninit();

pub(crate) fn key_enque(scancode: pc_keyboard::DecodedKey) {
    if let Ok(queue) = KEY_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}
