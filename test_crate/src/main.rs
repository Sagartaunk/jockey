use jockey_obfuscator::obfuscate;
#[obfuscate(0.6, unsafe_ok, allow_inline)]
fn main() {
    let a: i32 = 10;
    let b: i32 = 20;
    let c: i32 = a + b;
    println!("{}", c);
}
