use std::path::PathBuf;
use std::{env, fs};

const LC3TOOLS_SCRIPT: &'static str = "lc3tools_executor.sh";

fn main() -> std::io::Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::copy(LC3TOOLS_SCRIPT, out_dir.join(LC3TOOLS_SCRIPT))?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", LC3TOOLS_SCRIPT);

    Ok(())
}
