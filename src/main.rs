use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = changement::changement_main(args);
    println!("{}", result);
}
