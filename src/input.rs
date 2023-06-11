use std::collections::HashMap;

use winit::event::VirtualKeyCode;

#[derive(Default, Debug)]
pub struct Input {
    keyboard: HashMap<VirtualKeyCode, u32>,
}

impl Input {
    pub(crate) fn press(&mut self, keycode: VirtualKeyCode) {
        self.keyboard.insert(keycode, 0);
    }
    pub(crate) fn release(&mut self, keycode: VirtualKeyCode) {
        self.keyboard.insert(keycode, !0);
    }
    pub(crate) fn next_tick(&mut self) {
        self.keyboard.retain(|_, time| {
            *time = (*time).wrapping_add(1);
            *time != 0
        });
    }
    pub fn is_pressed(&self, keycode: VirtualKeyCode) -> bool {
        self.keyboard.get(&keycode).is_some_and(|&t| t != !0)
    }
    pub fn is_just_pressed(&self, keycode: VirtualKeyCode) -> bool {
        self.keyboard.get(&keycode).is_some_and(|&t| t == 0)
    }
    pub fn is_just_released(&self, keycode: VirtualKeyCode) -> bool {
        self.keyboard.get(&keycode).is_some_and(|&t| t == !0)
    }
}
