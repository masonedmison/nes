use bus::Bus;
use cartridge::Cartridge;
use cpu::CPU;


extern crate sdl2;

mod bus;
mod cpu;
mod utils;
mod debug;
mod cartridge;
mod ppu;

fn main() {
    let file_path = "./test_roms/cpu/nestest.nes";
    let cartridge = Cartridge::load(file_path).expect("Error loading file");
    let bus: Bus = todo!();
    let mut cpu = CPU::new(bus);

    cpu.load_cartridge(cartridge);
    cpu.run_debug()
}
