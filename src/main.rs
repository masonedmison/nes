use bus::Bus;
use cartridge::Cartridge;
use cpu::CPU;
use ppu::PPU;

extern crate sdl2;

mod bus;
mod cartridge;
mod cpu;
mod debug;
mod ppu;
mod utils;

fn main() {
    let file_path = "./test_roms/cpu/nestest.nes";
    let cartridge = Cartridge::load(file_path).expect("Error loading file");
    let ppu = PPU::new();
    let bus: Bus = Bus::new(ppu);
    let mut cpu = CPU::new(bus);

    cpu.load_cartridge(cartridge);
    todo!()
}
