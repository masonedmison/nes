const CPU_INTERNAL_RAM: usize = 2048;
const PAGE_SIZE: usize = 0xff;
// Zero page reserved for a number of special addressing modes
pub struct Bus {
    ram: [u8; CPU_INTERNAL_RAM],
    rom_bank1: [u8; 0x4000],
    rom_bank2: [u8; 0x4000],
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            ram: [0; CPU_INTERNAL_RAM],
            rom_bank1: [0; 0x4000],
            rom_bank2: [0; 0x4000],
        }
    }
    // TODO Hack: for now, just load bytes into both roms bank and 2
    pub fn load_rom(&mut self, bytes: [u8; 0x4000]) {
        self.rom_bank1 = bytes.clone();
        self.rom_bank2 = bytes;
    }
    // if 3 most significant bytes are 0, we take the first 11 digits which
    // gives the "effect" that we mirror the first 2kb onto addresses at 0x800-0x2000
    fn mirror(addr: u16) -> u16 {
        match addr {
            0x0..=0x1ff => addr & 0x7ff,
            0x2000..=0x3fff => (addr & 0xf) % 8,
            _ => addr,
        }
    }

    // Only considering cpu internal ram and simplified ROM for the time being.
    pub fn read_memory(&self, addr: u16) -> u8 {
        if addr >= 0x8000 {
            let offset = ((addr - 0x8000) % 0x4000) as usize;
            // TODO hard code "mapping logic" now...
            if addr < 0xC000 {
                self.rom_bank1[offset]
            } else {
                self.rom_bank2[offset]
            }
        } else {
            self.ram[Bus::mirror(addr) as usize]
        }
    }

    pub fn write_memory(&mut self, addr: u16, byte: u8) {
        // TODO Hack, only allow writes to memory below 0x8000
        if addr < 0x8000 {
            self.ram[Bus::mirror(addr) as usize] = byte
        }
    }
}
