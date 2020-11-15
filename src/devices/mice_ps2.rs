use crate::prelude::*;
#[derive(Copy, Clone)]
struct PS2MouseInternals {
    x: u32,
    y: u32,
}
_ezy_static! { PS2_MOUSE_INTERNALS_INSTANCE, PS2MouseInternals, PS2MouseInternals { x: 0, y: 0} }

#[derive(Debug)]
pub struct PS2Mouse;
impl PS2Mouse {
    pub fn init(&self) {}
}
impl crate::devices::mice::Mouse for PS2Mouse {
    fn get_x(&self) -> u32 {
        return PS2_MOUSE_INTERNALS_INSTANCE.get().x;
    }
    fn get_y(&self) -> u32 {
        return PS2_MOUSE_INTERNALS_INSTANCE.get().y;
    }
}
static STATE: AtomicU8 = AtomicU8::new(0);
lazy_static! {
    static ref MOUSE_DATA: Mutex<Vec<u8>> = Mutex::new(vec![0, 0, 0]);
}
pub fn handle_mouse_interrupt() {
    let da = MOUSE_DATA.get();
    match STATE.load(Ordering::Relaxed) {
        0 => {
            STATE.store(1, Ordering::Relaxed);
            da[0] = unsafe { inb(0x60) };
        }
        1 => {
            STATE.store(2, Ordering::Relaxed);
            da[1] = unsafe { inb(0x60) };
        }
        2 => {
            STATE.store(0, Ordering::Relaxed);
            da[2] = unsafe { inb(0x60) };
            let mut ii = PS2_MOUSE_INTERNALS_INSTANCE.get();
            ii.x = (ii.x as i32 + (da[0] as i8) as i32) as u32;
            ii.y = (ii.y as i32 + (da[1] as i8) as i32) as u32;
        }
        _ => unreachable!(),
    }
}
