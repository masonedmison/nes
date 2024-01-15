const CPU_INTERNAL_RAM: usize = 2048;
const PAGE_SIZE: usize = 0xff;
// Zero page reserved for a number of special addressing modes
pub struct Bus {
    ram: [u8; CPU_INTERNAL_RAM],
}

impl Bus {
    // if 3 most significant bytes are 0, we take the first 11 digits which
    // gives the "effect" that we mirror the first 2kb onto addresses at 0x800-0x2000
    fn mirror(addr: u16) -> u16 {
        if ((addr & !0x1fff) >> 13) == 0 {
            addr & 0x7ff
        } else {
            addr
        }
    }
    // Only considering cpu internal ram for the time being.
    pub fn read_memory(&self, addr: u16) -> u8 {
        self.ram[Bus::mirror(addr) as usize]
    }

    pub fn write_memory(&mut self, addr: u16, byte: u8) {
        self.ram[Bus::mirror(addr) as usize] = byte
    }
}
