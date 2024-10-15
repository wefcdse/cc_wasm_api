use std::{env, fs, path::Path};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    {
        let dest_path = Path::new(&out_dir).join("num_map.rs");
        fs::write(&dest_path, gen_str()).unwrap();
    }
    println!("cargo::rerun-if-changed=build.rs");
}

fn gen_str() -> String {
    let num = 512;
    let mut o = format!(r#"pub const NUM_MAP: [&str; {num}] = ["#);
    for i in 0..num {
        o += &format!(r#" "{i}", "#);
    }
    o += "];";
    o
}
