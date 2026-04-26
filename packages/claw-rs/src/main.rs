use std::env;
use std::fs;

mod translator;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Claw: No input file.");
        return;
    }
    
    let js_code = fs::read_to_string(&args[1]).expect("Failed to read JS file");
    let rust_code = translator::translate_to_rust(&js_code);
    
    println!("// Transpiled Rust code (Claw output):\n{}", rust_code);
}
