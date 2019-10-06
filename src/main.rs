type Addr = u16;
type Word = u16;



trait Memory {
    fn read_word(&self, addr: Addr) -> Word;
    fn write_word(&mut self, addr: Addr, word: Word);

    fn flush(&self) -> Result<(), ()>;
}

struct MemoryShim {
    memory: [Word; ((core::u16::MAX) / 2) as usize],
}

impl Default for MemoryShim {
    fn default() -> Self {
        Self {
            memory: [0u16; ((core::u16::MAX) / 2) as usize]
        }
    }
}

// impl MemoryShim {
//     fn foo(self) -> u16 {
//         0
//     }
// }

impl Memory for MemoryShim {
    fn read_word(&self, addr: Addr) -> Word {
        self.memory[addr as usize]
    }

    fn write_word(&mut self, addr: Addr, word: Word) {
        self.memory[addr as usize] = word;
    }

    fn flush(&self) -> Result<(), ()> { Ok(()) }
}

trait Control {
    fn set_pc(&mut self, addr: Addr);
    fn step(&mut self);

    fn write_word(&mut self, addr: Addr, word: Word);
}

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
