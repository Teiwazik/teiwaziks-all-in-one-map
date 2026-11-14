//! Prints the frame hashes of `megamap_engine_tests::golden`: run before and after a change, compare the output.

fn main() {
    let all = megamap_engine_tests::golden::run(|l| println!("{l}"));
    println!("all {all:08x}");
}
