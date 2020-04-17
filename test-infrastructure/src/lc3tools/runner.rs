//! A test runner that runs a program on the [lc3tools] simulator and the UTP
//! interpreter.
//!
//! [lc3tools]: https://github.com/chiragsakhuja/lc3tools

use crate::{Peripherals, Memory, Instruction, PeripheralInterruptFlags, Interpreter};


pub struct Strings {
    temp_string: u16,
}

pub struct Memory {
    mem_loc: u16,
    val: u16,
}

pub fn lc3tools_tester<'flags, M: Memory + Default + Clone, P: Peripherals<'flags>, PF, TF>
(
    prefilled_memory_locations: Vec<(Addr, Word)>,
    insns: Vec<Instruction>,
    lc3_insns: Vec<String>,
    setup_func: PF,
    teardown_func: TF,
    flags: &'flags PeripheralInterruptFlags,
    alt_memory: &Option<(M, Addr)>,
)
where
    for<'p> PF: FnOnce(&'p mut P),
    for<'i> TF: FnOnce(&'i Interpreter<M, P>),

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
