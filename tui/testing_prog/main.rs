use std::process::Command;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};


// fn get_state(output_lines: &str, registers: HashMap<&str, &str>){
    

#[derive(Debug, Clone)]
pub struct string_flags{
    temp_string: std::string::String,
    condition: bool,
}

fn filename_to_string(s: &str) -> io::Result<String> {
    let mut file = File::open(s)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s)
}
    
fn words_by_line<'a>(s: &'a str) -> Vec<Vec<&'a str>> {
    s.lines().map(|line| {
        line.split_whitespace().collect()
    }).collect()
}

fn find_memory(s: Vec<string_flags>, mem:&str) -> String{
    for items in s{
        if items.temp_string.contains(mem){
           return items.temp_string.to_owned();
        }
    }
    let ret: String = "Can't find".to_string();
    return ret;
}

fn main() {
   
    test_interpreter();


}

fn test_interpreter(){
// run the bash script by itself 
    

    // let output = Command::new("sh")
    //             .arg("-c")
    //             .arg("bash lc3tools/build/bin/lc3tools_executor.sh")
    //             .arg(insns_filename)
    //             .arg("file_test.txt")
    //             .output()
    //             .expect("failed to execute process");
    
    // let out = output.stdout;
    //  println!("{:?}",out);
    let mut global_vec_pc = Vec::new();
    let mut global_vec_psr = Vec::new();
    let mut global_vec_r0 = Vec::new();
    let mut global_vec_r4 = Vec::new();
    let mut global_vec_mem = Vec::new();
    
    //let file = File::open("./file_test.txt").unwrap();
    //let reader = BufReader::new(file);
    //let output_lines = reader.lines();
                            // .map(|l| l.expect("Could not parse line"))
                            // .collect();
    

    let mut lc3tools_registers: HashMap<&str, &str> = HashMap::new();
    let mut lc3tools_memory: HashMap<usize, usize> = HashMap::new();
    for pat in 0..65535 {
        lc3tools_memory.insert(pat, 0);
    }
    lc3tools_registers.insert(
        "r0",
        "0x0",
    );
    lc3tools_registers.insert(
        "r1",
        "0x0",
    );
    lc3tools_registers.insert(
        "r2",
        "0x0",
    );
    lc3tools_registers.insert(
        "r3",
        "0x0",
    );
    lc3tools_registers.insert(
        "r4",
        "0x0",
    );
    lc3tools_registers.insert(
        "r5",
        "0x0",
    );
    lc3tools_registers.insert(
        "r6",
        "0x0",
    );
    lc3tools_registers.insert(
        "r7",
        "0x0",
    );
    lc3tools_registers.insert(
        "pc",
        "0x0",
    );
    lc3tools_registers.insert(
        "psr",
        "0x0",
    );

   
    let filename = "file_test.txt";
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
   
    // Read the file line by line using the lines() iterator from std::io::BufRead.
    for (index, line_temp) in reader.lines().enumerate() {
        
        let line_loc = line_temp.unwrap(); // Ignore errors.
        //let mut line: std::string::String= String::from("default");
        // Show the line and its number.
        let pc_flag = line_loc.contains("PC: ");
        let psr_flag = line_loc.contains("PSR: ");
        let r0_flag = line_loc.contains("R0: ");
        let r4_flag = line_loc.contains("R4: ");
       // line = line.clone();
       let mem_flag = line_loc.contains(":");
            //line = line_loc;
       
        //println!("{:?}", line_loc);
        ;
        if pc_flag == true {
            
            global_vec_pc.push(string_flags{temp_string: line_loc, condition: pc_flag});
        }
        else if psr_flag == true {
           
            global_vec_psr.push(string_flags{temp_string: line_loc, condition: psr_flag});
        }
        else if r0_flag == true {
           // println!("{:?}", line_loc);
            global_vec_r0.push(string_flags{temp_string: line_loc, condition: r0_flag});
        }
        else if r4_flag == true {
            global_vec_r4.push(string_flags{temp_string: line_loc, condition: r4_flag});
        }
        else {
        	if index > 32{
            global_vec_mem.push(string_flags{temp_string: line_loc, condition: mem_flag});
        }
        }
      
    }
    
   let pc = global_vec_pc.pop().unwrap();    
   lc3tools_registers.insert("pc", pc.temp_string.split("PC: ").collect::<Vec<&str>>()[1]);
   
   let psr = global_vec_psr.pop().unwrap();    
   lc3tools_registers.insert("psr", psr.temp_string.split("PSR: ").collect::<Vec<&str>>()[1]);
   
   let r0 = global_vec_r0.pop().unwrap();    
   let registers0123 = r0.temp_string.split("R").collect::<Vec<&str>>();

    lc3tools_registers.insert("r0", registers0123[1].split(" ").collect::<Vec<&str>>()[1]);
    lc3tools_registers.insert("r1", registers0123[2].split(" ").collect::<Vec<&str>>()[1]);
    lc3tools_registers.insert("r2", registers0123[3].split(" ").collect::<Vec<&str>>()[1]);
    lc3tools_registers.insert("r3", registers0123[4].split(" ").collect::<Vec<&str>>()[1]);


   //println!("{:?}", registers0123); //r0.temp_string.split("R0: ").collect::<Vec<&str>>()[1].split(" ").collect::<Vec<&str>>());
   //lc3tools_registers.insert("r0", r0.temp_string.split("R0: ").collect::<Vec<&str>>()[1]).unwrap().split(" ").collect::<Vec<&str>>()[0];
   
  
   let r4 = global_vec_r4.pop().unwrap();
   let registers4567 = r4.temp_string.split("R").collect::<Vec<&str>>();
   lc3tools_registers.insert("r4", registers4567[1].split(" ").collect::<Vec<&str>>()[1]);
   lc3tools_registers.insert("r5", registers4567[2].split(" ").collect::<Vec<&str>>()[1]);
   lc3tools_registers.insert("r6", registers4567[3].split(" ").collect::<Vec<&str>>()[1]);
   lc3tools_registers.insert("r7", registers4567[4].split(" ").collect::<Vec<&str>>()[1]);
   
  
    for items in global_vec_mem{
        let res = items.temp_string.contains("0x");
        if res == true{
            let key= hex_string_to_integer(
            				 items.temp_string.split(" ").collect::<Vec<&str>>()[0]
            								  .split("x").collect::<Vec<&str>>()[1]
            								  .split(":").collect::<Vec<&str>>()[0]);
            let value = hex_string_to_integer(
            				 items.temp_string.split(" ").collect::<Vec<&str>>()[1]
            								  .split("x").collect::<Vec<&str>>()[1]);
            lc3tools_memory.insert(key as usize, value as usize);
            								//  .split(":").collect::<Vec<&str>>()[0]);
        }
       
    }

   

    //let iterator = global_vec_mem.iter();
    //iterator.



//    //println!("{:?}", registers4567);
   //lc3tools_registers.insert("r4", r4.temp_string.split("R4: ").collect::<Vec<&str>>()[1]);
   // println!("{:?}", find_memory(global_vec_mem, "0xF000").split(":").collect::<Vec<&str>>());
    
  
    // lc3tools_registers.insert(mem_val[0], mem_val[1]);

    //println!("{:?}", lc3tools_registers);



    // for l in lines {
    //     if l.contains("pc: ".to_string()) {
    //         let split_line: Vec<&str> = lines.split(' ').collect();
    //         lc3tools_registers.insert("pc", split_line[1]);
    //     }
    // }

    // for lines in output_lines {
    //     if(lines.contains("pc: ")){
    //         let split_line: Vec<&str> = lines.split(' ').collect();
    //         lc3tools_registers.insert("pc", split_line[1]);
    //     }
    // }


   //let mut lc3tools_memory: HashMap<&str, &str> = HashMap::new();
    
    for (key, value) in lc3tools_memory.into_iter() {
            println!("{} / {}", key, value);

    }
    


}

fn hex_string_to_integer(s: &str) -> u32 {
    //let s = "FFFF";
    let char_vec : Vec<char> = s.to_string().chars().collect();
    let mut ctr=3;
    let mut value: u32=0;
    for c in char_vec {
        
        
        
        match c {
            'F' =>{
                value += 15*u32::pow(16, ctr);
                //println!("{}", value);
            }
            'E' =>{
                value += 14*u32::pow(16, ctr);
            }
            'D' =>{
                value += 13*u32::pow(16, ctr);
            }
            'C' =>{
                value += 12*u32::pow(16, ctr);
            }
            'B' =>{
                value += 11*u32::pow(16, ctr);
            }
            'A' =>{
                value += 10*u32::pow(16, ctr);
            }
            '9' =>{
                value += 9*u32::pow(16, ctr);
            }
            '8' =>{
                value += 8*u32::pow(16, ctr);
            }
            
            '7' =>{
                value += 7*u32::pow(16, ctr);
            }
            '6' =>{
               value += 6*u32::pow(16, ctr);
            }
            
            '5' =>{
                value += 5*u32::pow(16, ctr);
            }
            '4' =>{
                value += 4*u32::pow(16, ctr);
            }
            '3' =>{
                value += 3*u32::pow(16, ctr);
            }
            '2' =>{
                value += 2*u32::pow(16, ctr);
            }
            '1' =>{
                value += 1*u32::pow(16, ctr);
            }
            '0' =>{
                
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







