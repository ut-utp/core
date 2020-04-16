use lc3_isa::{Addr, Instruction, Word};
use lc3_traits::memory::Memory;
use lc3_traits::peripherals::Peripherals;
use std::fs::{File, remove_file};
use std::io::prelude::*;
use std::io::BufReader;
use std::process::Command;
use std::io::Write;
use lc3_baseline_sim::interp::{PeripheralInterruptFlags, InstructionInterpreter,
    Interpreter, InterpreterBuilder, MachineState
};
extern crate rand;
use rand::Rng;
use core::convert::{TryFrom, TryInto};

use pretty_assertions::assert_eq;


pub struct strings{
    temp_string: u16,
    //val: std::string::String,
    //register: std::string::String,
}
pub struct memory{
    mem_loc: u16,
    val: u16,

}
fn hex_string_to_integer(s: String) -> u16 {
    //let s = "FFFF";
    let char_vec : Vec<char> = s.chars().collect();
    let mut ctr=5;
    let mut value: u16=0;
    for c in char_vec {
        match c {
            'F' =>{
                value += 15*u16::pow(16, ctr);
                //println!("{}", value);
            }
            'E' =>{
                value += 14*u16::pow(16, ctr);
            }
            'D' =>{
                value += 13*u16::pow(16, ctr);
            }
            'C' =>{
                value += 12*u16::pow(16, ctr);
            }
            'B' =>{
                value += 11*u16::pow(16, ctr);
            }
            'A' =>{
                value += 10*u16::pow(16, ctr);
            }
            '9' =>{
                value += 9*u16::pow(16, ctr);
            }
            '8' =>{
                value += 8*u16::pow(16, ctr);
            }

            '7' =>{
                value += 7*u16::pow(16, ctr);
            }
            '6' =>{
               value += 6*u16::pow(16, ctr);
            }

            '5' =>{
                value += 5*u16::pow(16, ctr);
            }
            '4' =>{
                value += 4*u16::pow(16, ctr);
            }
            '3' =>{
                value += 3*u16::pow(16, ctr);
            }
            '2' =>{
                value += 2*u16::pow(16, ctr);
            }
            '1' =>{
                value += 1*u16::pow(16, ctr);
            }
            '0' =>{
                value += 0;
            }
            _=>{

            }

        }
        if (ctr>0){
        ctr = ctr-1;
        }
    }
    value
}

pub fn lc3tools_tester<'a, M: Memory + Default + Clone, P: Peripherals<'a>, PF, TF>
(
    prefilled_memory_locations: Vec<(Addr, Word)>,
    insns: Vec<Instruction>,
    lc3_insns: Vec<String>,
    setup_func: PF,
    teardown_func: TF,
    flags: &'a PeripheralInterruptFlags,
    alt_memory: &Option<(M, Addr)>,
)
where
    for<'p> PF: FnOnce(&'p mut P),
    for<'p> TF: FnOnce(&'p Interpreter<M, P>),

    {
    let mut rng = rand::thread_rng();
    let n1: u8 = rng.gen();
    let file1_str = &format!("./test_lc3_{}.txt", n1);

    let file1 = File::create(file1_str.to_string());

    let mut insns_lc3tools = Vec::<String>::new();
    let num_steps = lc3_insns.len();
    insns_lc3tools.push(format!("{}", num_steps).to_string());
    insns_lc3tools.push(".orig x3000".to_string());
    for lc3_insn in lc3_insns {
        insns_lc3tools.push(lc3_insn);
    }
    insns_lc3tools.push(".end".to_string());
    let mut string_insns = insns_lc3tools.join("\n");

    file1.unwrap().write_all(string_insns.as_bytes());

    let outfile = &format!("lc3tools_output_{}.txt", n1);
    let mut output_command = Command::new("bash").arg("./lc3tools_executor.sh").arg(file1_str).arg(&format!("{}", n1)).spawn().unwrap();

    let _result = output_command.wait().unwrap();

    let mut file = File::open(outfile).expect("Can't open File");
    let reader = BufReader::new(file);

    let mut vec_mem = Vec::<memory>::new();
    let mut vec_registers = Vec::<strings>::new();

    let mut pc = String::new();
    let mut pc = String::new();
    let mut psr = String::new();
    let mut cc = String::new();
    let mut mcr = String::new();


    let mut addr = 0x3000;

    let mut interp: Interpreter<M, P> = if let Some((mem, addr)) = alt_memory {
        let mut int: Interpreter<M, P> = InterpreterBuilder::new()
            .with_defaults()
            .with_memory(mem.clone())
            .build();

        int.reset();
        int.set_pc(*addr);

        int
    } else {
        let mut int = Interpreter::<M, P>::default();

        int.reset();
        int.set_pc(addr);

        int
    };

    interp.init(flags);

    // Run the setup func:
    setup_func(&mut *interp);

    // Prefill the memory locations:
    for (addr, word) in prefilled_memory_locations.iter() {
        // Crashes on ACVs! (they should not happen at this point)
        interp.set_word(*addr, *word).unwrap()
    }

    for insn in insns {

        interp.set_word_unchecked(addr, insn.into());

        addr += 1;
    }

    for _ in 0..num_steps {
        interp.step();
    }


    for line_temp in reader.lines() {
        let line = line_temp.unwrap();
        //println!("{:?}", line);

        if !line.is_empty(){

            let address = &line[0..2];
            if address == "0x" {

                let value: u16 = hex_string_to_integer(line.split(" ").collect::<Vec<&str>>()[1].to_string());

                let mem_location: u16 = hex_string_to_integer(line.split(" ").collect::<Vec<&str>>()[0].to_string().replace(":", ""));
                //println!("{:?}", mem_location);
                 vec_mem.push(memory{mem_loc: mem_location, val: value});

            }
            else if address == "PC" {
                pc = line.split(" ").collect::<Vec<&str>>()[1].to_string();



            }
            else if address == "PSR" {
                psr = line.split(" ").collect::<Vec<&str>>()[1].to_string();
            }
            else if address == "CC" {
                cc = line.split(" ").collect::<Vec<&str>>()[1].to_string();
            }
            else if address == "MCR" {
                mcr = line.split(" ").collect::<Vec<&str>>()[1].to_string();
            }

            if line.contains("R0:"){
                vec_registers = Vec::<strings>::new();
                let registers0123 = line.split("R").collect::<Vec<&str>>();
                vec_registers.push(strings{temp_string: hex_string_to_integer(registers0123[1].split(" ").collect::<Vec<&str>>()[1].to_string())});
                vec_registers.push(strings{temp_string: hex_string_to_integer(registers0123[2].split(" ").collect::<Vec<&str>>()[1].to_string())});
                vec_registers.push(strings{temp_string: hex_string_to_integer(registers0123[3].split(" ").collect::<Vec<&str>>()[1].to_string())});
                vec_registers.push(strings{temp_string: hex_string_to_integer(registers0123[4].split(" ").collect::<Vec<&str>>()[1].to_string())});
            } else if line.contains("R4:"){
                let registers4567 = line.split("R").collect::<Vec<&str>>();
                vec_registers.push(strings{temp_string: hex_string_to_integer(registers4567[1].split(" ").collect::<Vec<&str>>()[1].to_string())});
                vec_registers.push(strings{temp_string: hex_string_to_integer(registers4567[2].split(" ").collect::<Vec<&str>>()[1].to_string())});
                vec_registers.push(strings{temp_string: hex_string_to_integer(registers4567[3].split(" ").collect::<Vec<&str>>()[1].to_string())});
                vec_registers.push(strings{temp_string: hex_string_to_integer(registers4567[4].split(" ").collect::<Vec<&str>>()[1].to_string())});
            }
        }

    }


     // Check registers:
     for (idx, r) in vec_registers.iter().enumerate() {
            let reg_word = r.temp_string;
            let val = interp.get_register((idx as u8).try_into().unwrap());
            assert_eq!(
                reg_word,
                val,
                "Expected R{} to be {:?}, was {:?}",
                idx,
                reg_word,
                val,
            );

    }

    // Check memory:
    for object in vec_mem.iter() {
        let addr = object.mem_loc;
        let word = object.val;
        let val = interp.get_word_unchecked(addr);
        if addr > 768{ // check memory after all the ACVS... need to find a workaround...
            assert_eq!(
                word, val,
                "Expected memory location {:#04X} to be {:#04X}",
                addr, val
            );
        }

    }

    // Run the teardown func:
    teardown_func(&interp);

    remove_file(outfile);




}

#[inline]
pub fn interp_test_runner<'flags, M: Memory + Default + Clone, P: Peripherals<'flags>, PF, TF>
(
    prefilled_memory_locations: Vec<(Addr, Word)>,
    insns: Vec<Instruction>,
    num_steps: Option<usize>,
    regs: [Option<Word>; 8],
    pc: Option<Addr>,
    memory_locations: Vec<(Addr, Word)>,
    setup_func: PF,
    teardown_func: TF,
    flags: &'flags PeripheralInterruptFlags,
    alt_memory: Option<(M, Addr)>,
    alt_peripherals: Option<P>,
)
where
    for<'p> PF: FnOnce(&'p mut P),
    for<'i> TF: FnOnce(&'i Interpreter<'flags, M, P>), // Note: we could pass by value
                                                       // since this is the last thing
                                                       // we do.
{
    let mut addr = 0x3000;

    let interp_builder = InterpreterBuilder::new().with_defaults();

    let interp_builder = if let Some(peripherals) = alt_peripherals {
        interp_builder.with_peripherals(peripherals)
    } else {
        interp_builder
    };

    let mut interp: Interpreter<M, P> = if let Some((mem, addr)) = alt_memory {
        let mut int: Interpreter<M, P> = interp_builder
            .with_memory(mem)
            .build();

        int.reset();
        int.set_pc(addr);

        int
    } else {
        let mut int = interp_builder.build();

        int.reset();
        int.set_pc(addr);

        int
    };

    interp.init(flags);

    // Run the setup func:
    setup_func(&mut *interp);

    // Prefill the memory locations:
    for (addr, word) in prefilled_memory_locations.iter() {
        // Crashes on ACVs! (they should not happen at this point)
        interp.set_word(*addr, *word).unwrap()
    }

    for insn in insns {
        // let enc = Into::<u16>::into(insn);
        // println!("{:?}", insn);
        // println!("{:#04X} -> {:?}", enc, Instruction::try_from(enc));
        interp.set_word_unchecked(addr, insn.into());
        // println!(
        //     "{:?}",
        //     Instruction::try_from(interp.get_word_unchecked(addr))
        // );

        addr += 1;
    }

    if let Some(num_steps) = num_steps {
        for _ in 0..num_steps {
            // println!("step: x{0:4X}", interp.get_pc());
            interp.step();
        }
    } else {
        while let MachineState::Running = interp.step() { }
    }

    // Check PC:
    if let Some(expected_pc) = pc {
        let actual_pc = interp.get_pc();
        assert_eq!(
            expected_pc,
            actual_pc,
            "Expected PC = {:#04X}, got {:#04X}",
            expected_pc,
            actual_pc
        );
    }


    // Check registers:
    for (idx, r) in regs.iter().enumerate() {
        if let Some(reg_word) = r {
            let val = interp.get_register((idx as u8).try_into().unwrap());
            assert_eq!(
                *reg_word,
                val,
                "Expected R{} to be {:?}, was {:?}",
                idx,
                *reg_word,
                val,
            );
        }
    }

    // Check memory:
    for (addr, word) in memory_locations.iter() {
        let val = interp.get_word_unchecked(*addr);
        assert_eq!(
            *word, val,
            "Expected memory location {:#04X} to be {:#04X}",
            *word, val
        );
    }

    // Run the teardown func:
    teardown_func(&interp);
}
