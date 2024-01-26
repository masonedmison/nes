use core::panic;
use std::{error::Error, fs};

// NES follow by MS-DOS end of file
const NES_TAG: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];
const CHR_ROM_SIZE: usize = 0x2000;
const PRG_ROM_SIZE: usize = 0x4000;

pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
}
pub struct Cartridge {
    pub prgrom: Vec<u8>,
    pub chrrom: Vec<u8>,
    pub mirroring: Mirroring,
    pub mapper: u8,
}

impl Cartridge {
    pub fn load(path: &str) -> Result<Cartridge, Box<dyn Error>> {
        let bytes = fs::read(path)?;
        let header = &bytes[0..=15];
        let flag6 = header[6];
        let flag7 = header[7];

        // validation
        if header[0..4] != NES_TAG {
            panic!("File is not in the iNES file format.")
        }
        let ines_version = (flag7 >> 2) & 0b11;
        if ines_version != 0 {
            panic!("Only iNES1 version is supported.")
        }
        // ********

        let four_screen = (flag6 >> 3) & 0b1 == 1;
        let vertical = flag6 & 0b1 == 1;
        let mirroring = match (four_screen, vertical) {
            (true, _) => Mirroring::FourScreen,
            (_, true) => Mirroring::Vertical,
            (_, false) => Mirroring::Horizontal,
        };

        let has_trainer = (flag6 >> 2) & 0b1 == 0b1;
        let prgrom_start = (if has_trainer { 512 } else { 0 } + 16) as usize;
        let prgrom_size = PRG_ROM_SIZE * (header[4] as usize);
        let chrrom_start = prgrom_start + prgrom_size;
        let chrrom_size = CHR_ROM_SIZE * (header[5] as usize);

        let prgrom: Vec<u8> = bytes[prgrom_start..(prgrom_start + prgrom_size)].to_vec();
        let chrrom = bytes[chrrom_start..(chrrom_start + chrrom_size)].to_vec();

        let mapper = flag7 & 0b11110000 | flag6 >> 4;

        Ok(Cartridge {
            prgrom: prgrom,
            chrrom,
            mirroring,
            mapper,
        })
    }
}
