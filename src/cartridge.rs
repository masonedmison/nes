use std::{error::Error, fs};
pub struct Cartridge {
    pub bytes: [u8; 0x4000],
}

impl Cartridge {
    pub fn load(path: &str) -> Result<Cartridge, Box<dyn Error>> {
        let bytes = fs::read(path)?;
        let mut buff = [0; 0x4000];
        let truncated = &bytes[0x10..0x4010];
        truncated
            .iter()
            .enumerate()
            .for_each(|(idx, byte)| buff[idx] = *byte);
        Ok(Cartridge { bytes: buff })
    }
}
