pub struct PPUBus {
    chr_rom: [u8; 0x1ff],
    vram: [u8; 2048],
    palette_table: [u8; 32]
}

impl PPUBus {
  pub fn new() -> PPUBus {
    PPUBus {
      chr_rom: [0; 0x1ff],
      vram: [0; 2048],
      palette_table: [0; 32],
    }
  }
  pub fn load_chr_rom(&mut self, chr_rom: [u8; 0x1ff]) {
    self.chr_rom = chr_rom
  }
  pub fn read_memory(&self, addr: u16) -> u8 {
      match addr {
        0x00..=0x1ff => self.chr_rom[addr as usize],
        0x2000..=0x2fff => self.vram[(addr - 0x2000) as usize],
        // mirror down
        0x3000..=0x3eff => self.vram[(addr - 0x3000) as usize],
        0x3f00..=0x3fff => {
            let offset = (0xff & addr) % 32;
            self.palette_table[offset as usize]
        } 
        _ => panic!("Only 14 bits can be addressed")
      }
  }
  pub fn write_memory(&mut self, addr: u16, value: u8) {
    // TODO for now, only allow writes to vram
    match addr {
        0x00..=0x1ff => self.chr_rom[addr as usize] = value,
        0x2000..=0x2fff => self.vram[(addr - 0x2000) as usize] = value,
        0x3000..=0x3eff => self.vram[(addr - 0x3000) as usize] = value,
        0x3f00..=0x3fff => {
            let offset = (0xff & addr) % 32;
            self.palette_table[offset as usize] = value
        } 
        _ => panic!("can only write to vram (0x2000..0x2fff")
    }
  }
}
