use super::{
    ppubus::PPUBus,
    registers::{OAMADDR, OAMDATA, PPUADDR, PPUCTRL, PPUDATA, PPUMASK, PPUSCROLL, PPUSTATUS},
};

// TODO check how we are handling Mirroring - this might mean we need to
// properly implement how we parse nes files...
pub struct PPU {
    bus: PPUBus,
    oam: [u8; 64 * 4],
    // internal registers
    w: bool, /* false = hi/x coord, true = low/y coord */
    // IO mapped registers
    ppuctrl: PPUCTRL,
    ppumask: PPUMASK,
    ppustatus: PPUSTATUS,
    oamaddr: OAMADDR,
    oamdata: OAMDATA,
    ppuscroll: PPUSCROLL,
    ppuaddr: PPUADDR,
    ppudata: PPUDATA,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            bus: PPUBus::new(),
            oam: [0; 64 * 4],
            w: false,
            ppuctrl: PPUCTRL::new(),
            ppumask: PPUMASK::new(),
            ppustatus: PPUSTATUS::new(),
            oamaddr: OAMADDR ( 0 ),
            oamdata: OAMDATA ( 0 ),
            ppuscroll: PPUSCROLL::new(),
            ppuaddr: PPUADDR::new(),
            ppudata: PPUDATA ( 0 ),
        }
    }
    // TODO In general, we aren't handling any of the tricky
    // edge cases mentioned on the Registers NESDev page
    pub fn write_ppu_ctrl(&mut self, data: u8) {
        self.ppuctrl.update(data)
    }
    pub fn write_ppumask(&mut self, data: u8) {
        self.ppumask.update(data)
    }
    pub fn read_ppustatus(&mut self) -> u8 {
        self.ppustatus.set(PPUSTATUS::VBLANK_START, false);
        self.w = false;
        self.ppustatus.bits()
    }
    pub fn write_oamaddr(&mut self, data: u8) {
        self.oamaddr.0 = data
    }
    pub fn read_oamdata(&self) -> u8 {
        self.oamdata.0
    }
    pub fn write_oamdata(&mut self, data: u8) {
        self.oamdata.0 = data;
        self.oamaddr.0 += 1
    }
    pub fn write_ppuscroll(&mut self, data: u8) {
        self.ppuscroll.update(data, self.w);
        self.w = !self.w;
    }
    pub fn write_ppuaddr(&mut self, data: u8) {
        self.ppuaddr.update(data, self.w);
        self.w = !self.w;
    }
    pub fn increment_ppu_addr(&mut self) {
        self.ppuaddr.increment_by(self.ppuctrl.contains(PPUCTRL::VRAM_ADDR_INCR))
    }
    // TODO ignoring the edge case where a read
    // is issued against an address between 0x3f00..0x3fff
    pub fn read_ppudata(&mut self) -> u8 {
        let read = self.ppudata.0;
        self.ppudata.0 = self.bus.read_memory(self.ppuaddr.get());
        self.increment_ppu_addr();
        read
    }
    pub fn write_ppudata(&mut self, data: u8) {
        self.bus.write_memory(self.ppuaddr.get(), data);
        self.increment_ppu_addr()
    }
    pub fn write_dma(&mut self, bytes: &[u8]) {
        (self.oamaddr.0..=255).zip(bytes).for_each(|(idx, byte)| {
            self.oam[idx as usize] = *byte
        })
    }

}