use crate::cartridge::Mirroring;

pub struct PPUBus {
    chr_rom: [u8; 0x1ff],
    name_tables: [u8; 0x800],
    palette_table: [u8; 32],
    mirroring: Mirroring,
}

impl PPUBus {
    pub fn new() -> PPUBus {
        PPUBus {
            chr_rom: [0; 0x1ff],
            name_tables: [0; 2048],
            palette_table: [0; 32],
            mirroring: Mirroring::Horizontal,
        }
    }
    pub fn load_chr_rom(&mut self, chr_rom: [u8; 0x1ff], mirroring: Mirroring) {
        self.chr_rom = chr_rom;
        self.mirroring = mirroring
    }

    /*
     Mirroring schemes:
     Addresses
     [0x2000] [0x2400]
     [0x2800] [0x2c00]

     Reality
     [A] [B]

     Horizontal
     [A] [a]
     [B] [b]

     Vertical
     [A] [B]
     [a] [b]
    */
    fn mirror_to_nametable_addr(addr: u16, mirroring: &Mirroring) -> u16 {
        let base = match addr {
            0x2000..=0x2fff => addr - 0x2000,
            0x3000..=0x3eff => addr - 0x3000,
            _ => panic!("Attemping to mirror down address outside of addressable nametable range."),
        };
        let (table, idx) = (base / 0x400, base % 0x400);
        match (table, mirroring) {
            (0, _) => base,
            (1 | 3, Mirroring::Vertical) => 0x400 + idx,
            (2, Mirroring::Vertical) => idx,
            (1, Mirroring::Horizontal) => idx,
            (2 | 3, Mirroring::Horizontal) => 0x400 + idx,
            (_, Mirroring::FourScreen) => todo!(),
            _ => panic!("Inconceivable!"),
        }
    }
    pub fn read_memory(&self, addr: u16) -> u8 {
        match addr {
            0x00..=0x1ff => self.chr_rom[addr as usize],
            0x2000..=0x3eff => {
                let addr = PPUBus::mirror_to_nametable_addr(addr, &self.mirroring) as usize;
                self.name_tables[addr]
            }
            0x3f00..=0x3fff => {
                let offset = (0xff & addr) % 32;
                self.palette_table[offset as usize]
            }
            _ => panic!("Only 14 bits can be addressed"),
        }
    }
    pub fn write_memory(&mut self, addr: u16, value: u8) {
        // TODO for now, only allow writes to name_tables
        match addr {
            0x00..=0x1ff => self.chr_rom[addr as usize] = value,
            0x2000..=0x2fff => self.name_tables[(addr - 0x2000) as usize] = value,
            0x3000..=0x3eff => self.name_tables[(addr - 0x3000) as usize] = value,
            0x3f00..=0x3fff => {
                let offset = (0xff & addr) % 32;
                self.palette_table[offset as usize] = value
            }
            _ => panic!("can only write to to addresses 0x00..0x3f00."),
        }
    }
}

#[cfg(test)]
mod ppubus_test {
    use crate::cartridge::Mirroring;

    use super::PPUBus;

    #[test]
    fn test_mirror_to_nametable_addr() {
        let vertical0 = PPUBus::mirror_to_nametable_addr(0x2005, &Mirroring::Vertical);
        assert_eq!(vertical0, 0x05, "actual: {:#x}", vertical0);

        let vertical1 = PPUBus::mirror_to_nametable_addr(0x2405, &Mirroring::Vertical);
        assert_eq!(vertical1, 0x405, "actual: {:#x}", vertical1);

        let vertical2 = PPUBus::mirror_to_nametable_addr(0x2805, &Mirroring::Vertical);
        assert_eq!(vertical2, 0x05, "actual: {:#x}", vertical2);

        let vertical3 = PPUBus::mirror_to_nametable_addr(0x2c05, &Mirroring::Vertical);
        assert_eq!(vertical3, 0x405, "actual: {:#x}", vertical3);

        let horiztonal0 = PPUBus::mirror_to_nametable_addr(0x2005, &Mirroring::Horizontal);
        assert_eq!(horiztonal0, 0x05, "actual: {:#x}", horiztonal0);

        let horiztontal1 = PPUBus::mirror_to_nametable_addr(0x2405, &Mirroring::Horizontal);
        assert_eq!(horiztontal1, 0x05, "actual: {:#x}", horiztontal1);

        let horizontal2 = PPUBus::mirror_to_nametable_addr(0x2805, &Mirroring::Horizontal);
        assert_eq!(horizontal2, 0x405, "actual: {:#x}", horizontal2);

        let horizontal3 = PPUBus::mirror_to_nametable_addr(0x2c05, &Mirroring::Horizontal);
        assert_eq!(horizontal3, 0x405, "actual: {:#x}", horizontal3);
    }
}
