use super::{Addr, Word};

pub trait Control {
    fn set_pc(&mut self, addr: Addr);
    fn step(&mut self);

    fn write_word(&mut self, addr: Addr, word: Word);
}
