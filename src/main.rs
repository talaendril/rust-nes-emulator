use cpu::CPU;

mod cpu;
mod opcode;

fn main() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
}
