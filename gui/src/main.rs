use milton_core::tms1100::Tms1100;

fn main() {
    println!("Hello, Milton!");

    let mut tms = Tms1100::new();

    tms.rom.data[0] = 0x78;

    for _ in 0..(2 * 6) {
        tms.clock();
    }

    println!("The value of the accumlator is: {}", tms.a);
}
