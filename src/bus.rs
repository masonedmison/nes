use crate::ppu::PPU;

const CPU_INTERNAL_RAM: usize = 2048;
const PAGE_SIZE: usize = 0xff;
// Zero page reserved for a number of special addressing modes
pub struct Bus {
    ram: [u8; CPU_INTERNAL_RAM],
    rom_bank1: [u8; 0x4000],
    rom_bank2: [u8; 0x4000],
    ppu: PPU,
}

impl Bus {
    pub fn new(ppu: PPU) -> Bus {
        Bus {
            ram: [0; CPU_INTERNAL_RAM],
            rom_bank1: [0; 0x4000],
            rom_bank2: [0; 0x4000],
            ppu,
        }
    }
    // TODO Hack: for now, just load bytes into both roms bank and 2
    pub fn load_rom(&mut self, bytes: [u8; 0x4000]) {
        self.rom_bank1 = bytes.clone();
        self.rom_bank2 = bytes;
    }

    // TODO do we actually need a mutable reference here?
    fn read_io_registers(&mut self, reg: u8) -> u8 {
        match reg {
            0x2 => self.ppu.read_ppustatus(),
            0x4 => self.ppu.read_oamdata(),
            0x7 => self.ppu.read_ppudata(),
            _ => panic!(
                "Attempting to read a non-readable resgister: {:#x}",
                reg as usize + 0x2000
            ),
        }
    }
    fn write_io_registers(&mut self, reg: u8, data: u8) {
        match reg {
            0x0 => self.ppu.write_ppu_ctrl(data),
            0x1 => self.ppu.write_ppumask(data),
            0x3 => self.ppu.write_oamaddr(data),
            0x4 => self.ppu.write_oamdata(data),
            0x5 => self.ppu.write_ppuscroll(data),
            0x6 => self.ppu.write_ppuaddr(data),
            0x7 => self.ppu.write_ppudata(data),
            _ => panic!(
                "Attempting to write a non-writeable resgister: {:#x}",
                reg as usize + 0x2000
            ),
        }
    }
    fn oamdma(&mut self, page: u8) {
        let addrs = ((page as u16) << 8)..((page as u16) << 8 | 0xff);
        let bytes: Vec<u8> = addrs
            .map(|addr| match addr {
                0x0..=0x1ff => self.ram[(addr & 0x7ff) as usize],
                0x8000..=0xffff => self.read_rom(addr),
                _ => panic!(
                    "Can only oamdma internal ram and cartridge rom. Bad address: {:#x}",
                    addr
                ),
            })
            .collect();
        self.ppu.write_dma(&bytes)
    }

    fn read_rom(&self, addr: u16) -> u8 {
        let offset = ((addr - 0x8000) % 0x4000) as usize;
        if addr < 0xC000 {
            self.rom_bank1[offset]
        } else {
            self.rom_bank2[offset]
        }
    }

    // Only considering cpu internal ram and simplified ROM for the time being.
    pub fn read_memory(&mut self, addr: u16) -> u8 {
        if addr >= 0x8000 {
            self.read_rom(addr)
        } else {
            match addr {
                0x0..=0x1ff => {
                    let mirrored = (addr & 0x7ff) as usize;
                    self.ram[mirrored]
                }
                0x2000..=0x3fff => {
                    let mirrored = (addr & 0xf) % 8;
                    self.read_io_registers(mirrored as u8)
                }
                _ => self.ram[addr as usize],
            }
        }
    }

    pub fn write_memory(&mut self, addr: u16, byte: u8) {
        match addr {
            0x0..=0x1ff => {
                let mirrored = (addr & 0x7ff) as usize;
                self.ram[mirrored] = byte
            }
            0x2000..=0x3fff => {
                let mirrored = (addr & 0xf) % 8;
                self.write_io_registers(mirrored as u8, byte)
            }
            // TODO There will be more registers here eventually, only accounting for
            // oamdma at the moment.
            0x4014 => self.oamdma(byte),
            _ => self.ram[addr as usize] = byte,
        }
    }
}
