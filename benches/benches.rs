use criterion::{criterion_group, criterion_main, Criterion};
use emulator_6502::*;
use std::env::var;
use std::fs::*;
use std::io::*;
use std::path::PathBuf;

struct BasicRam {
    ram: Box<[u8; u16::max_value() as usize + 1]>,
    complete: bool,
}

impl BasicRam {
    fn load_program(&mut self, start: usize, data: &mut Vec<u8>) {
        self.ram[start..start + data.len()].clone_from_slice(data);
    }
}

impl Interface6502 for BasicRam {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0xfffe..=0xffff => {
                self.complete = true; //If brk has been called the test is complete
                0xff //Keep program in an infinite break loop until test terminates
            }
            _ => self.ram[address as usize],
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        self.ram[address as usize] = data
    }
}

/// Function for loading test programs
fn load_test(ram: &mut BasicRam, file_name: &str) -> Result<()> {
    let root_dir = &var("CARGO_MANIFEST_DIR").expect("$CARGO_MANIFEST_DIR");
    let mut source_path = PathBuf::from(root_dir);
    source_path.push("tests/bins");
    source_path.push(file_name);

    let mut file = File::open(source_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    ram.load_program(0x0400, &mut buffer);

    Ok(())
}

fn bench_test() {
    let mut ram = BasicRam {
        ram: Box::new([0; u16::max_value() as usize + 1]),
        complete: false,
    };
    load_test(&mut ram, "6502_bench.bin").unwrap();

    let mut cpu = MOS6502::new_start(0x400);
    while !ram.complete {
        cpu.cycle(&mut ram);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Loop Bench", |b| b.iter(bench_test));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
