type Addr = u16;
type Word = u16;

mod control;
mod memory;
mod peripherals;

mod isa;


use memory::Memory;
use control::Control;

struct Interpreter<M: Memory> {
    regs: [Word; 8],
    pc: Word,
    psr: Word,
    mem: M
}

impl<M: Memory> Control for Interpreter<M> {
    fn set_pc(&mut self, addr: Addr) {
        self.pc = addr;
    }

    fn step(&mut self) {
        unimplemented!()
    }

    fn write_word(&mut self, addr: Addr, word: Word) {
        // self.mem.foo();
        self.mem.write_word(addr, word);
    }
}




// fn foo(foo: impl Memory) -> () {
//     foo.flush();
// }

// fn foo2<M: Memory>(foo: M) -> () {
//     foo.flush();
// }



fn main() {
    println!("Hello, world!");

    // let i = Interpreter::<MemoryShim> {

    // };

    // i.mem.
}
