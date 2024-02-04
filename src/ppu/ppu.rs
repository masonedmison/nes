use super::{
    frame::Frame,
    palette::SYSTEM_PALLETE,
    ppubus::{PPUBus, BACKGROUND_COLOR},
    registers::{OAMADDR, OAMDATA, PPUADDR, PPUCTRL, PPUDATA, PPUMASK, PPUSCROLL, PPUSTATUS},
};

#[derive(Default)]
struct InternalRegisters {
    // coarse coordinates track x and y coord (at the tile level, e.g. 8 X 8)
    // for the current tile
    coarse_col: u16,
    coarse_row: u16,
    // fine coordinates track x and y coordinate at the pixel level
    fine_col: u8,
    fine_row: u8,
    nt_select: u8,
    w: bool,
}

pub struct PPU {
    bus: PPUBus,
    curr_frame: Frame,
    oam: [u8; 64 * 4],
    // IO mapped registers
    ppuctrl: PPUCTRL,
    ppumask: PPUMASK,
    ppustatus: PPUSTATUS,
    oamaddr: OAMADDR,
    oamdata: OAMDATA,
    ppuscroll: PPUSCROLL,
    ppuaddr: PPUADDR,
    ppudata: PPUDATA,
    // ********
    nmi_pin: bool,
    cycles: usize,
    scanline: u16,
    internal_reg: InternalRegisters,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            bus: PPUBus::new(),
            curr_frame: Frame::new(),
            oam: [0; 64 * 4],
            ppuctrl: PPUCTRL::new(),
            ppumask: PPUMASK::new(),
            ppustatus: PPUSTATUS::new(),
            oamaddr: OAMADDR(0),
            oamdata: OAMDATA(0),
            ppuscroll: PPUSCROLL::new(),
            ppuaddr: PPUADDR::new(),
            ppudata: PPUDATA(0),
            nmi_pin: false,
            cycles: 0,
            scanline: 0,
            internal_reg: Default::default(),
        }
    }
    pub fn poll_generate_nmi(&self) -> bool {
        self.nmi_pin
    }
    pub fn clear_generate_nmi(&mut self) {
        self.nmi_pin = false
    }
    fn fetch_chr_row(&self, addr: u16) -> (u8, u8) {
        (
            self.bus.read_memory(addr as u16),
            self.bus.read_memory((addr as u16) + 8),
        )
    }
    /**
     * Increments coarse coordinates
     * Stores fetched data
     */
    fn fetch_bg_tile_row(&mut self) -> usize {
        let nt_addr = self.internal_reg.coarse_row * 32 + self.internal_reg.coarse_col;
        let base_nt: u16 = match self.ppuctrl.get_base_nt() {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2c00,
            _ => panic!(),
        };
        let chr_idx = self.bus.read_memory(base_nt.wrapping_add(nt_addr));
        let base_chr = if self.ppuctrl.contains(PPUCTRL::BACKGROUND_PATTERN_TABLE) {
            0x1000
        } else {
            0
        };

        let attr_idx =
            ((self.internal_reg.coarse_row / 4) * 8) + (self.internal_reg.coarse_col / 4);
        let attr = self.bus.read_memory(base_nt + 0x3c0 + attr_idx);

        let palette_choice = {
            let palette_idx = match (
                self.internal_reg.coarse_col % 4 / 2,
                self.internal_reg.coarse_row % 4 / 2,
            ) {
                (0, 0) => attr & 0b11,
                (1, 0) => (attr >> 2) & 0b11,
                (0, 1) => (attr >> 4) & 0b11,
                (1, 1) => (attr >> 6) & 0b11,
                _ => panic!(),
            } as u16;
            match palette_idx {
                0 => 0x3f01,
                1 => 0x3f05,
                2 => 0x3f09,
                3 => 0x3f0d,
                _ => panic!(),
            }
        };

        let palette: (u8, u8, u8, u8) = (
            self.bus.read_memory(BACKGROUND_COLOR as u16),
            self.bus.read_memory(palette_choice),
            self.bus.read_memory(palette_choice + 1),
            self.bus.read_memory(palette_choice + 2),
        );

        // lo_plane controls bit 0 and hi_plane bit 1
        let (lo_plane, hi_plane) = self.fetch_chr_row(base_chr + chr_idx as u16);

        (0..=7).rev().for_each(|n| {
            let palette_idx = ((hi_plane >> n) & 1) << 1 | ((lo_plane >> n) & 1);
            let rgb = SYSTEM_PALLETE[match palette_idx {
                0 => palette.0,
                1 => palette.1,
                2 => palette.2,
                3 => palette.3,
                _ => panic!(),
            } as usize];
            self.curr_frame.set_pixel(
                self.internal_reg.fine_col as u8,
                self.internal_reg.fine_row as u8,
                rgb,
            );
            self.internal_reg.fine_col += 1
        });

        if self.internal_reg.coarse_col == 31 {
            self.internal_reg.fine_col = 0;
            self.internal_reg.coarse_col = 0;

            if self.internal_reg.fine_row % 8 == 0 {
                self.internal_reg.coarse_row += 1;
            }

            self.internal_reg.fine_row += 1;
        } else {
            self.internal_reg.coarse_col += 1
        }

        // TODO going to leave this coarse grained for now
        // and just treat this function as an atomic operation
        8
    }

    // TODO not yet considering odd/even cycle skips
    pub fn tick(&mut self, cycles: usize) {
        let mut remaining = cycles;
        while remaining > 0 {
            if self.cycles >= 340 {
                // if we are at the end of scanline 261
                // set scanline back to 0 to loop again
                if self.scanline == 261 {
                    self.scanline = 0
                } else {
                    self.scanline += 1;

                    // if we are entering scanline 241 and ppuctrl
                    // has the GENERATE_NMI flag set, it's nmi time baby
                    if self.scanline == 241 && self.ppuctrl.contains(PPUCTRL::GENERATE_NMI) {
                        self.ppustatus.set(PPUSTATUS::VBLANK_START, true);
                        self.nmi_pin = true
                    }

                    // if we are enterining scanline 261, toggle nmi_pin
                    // and we are no longer in vblank
                    if self.scanline == 261 {
                        self.ppustatus.set(PPUSTATUS::VBLANK_START, false);
                        self.nmi_pin = false
                    }
                }

                match self.scanline {
                    0 => self.cycles += 1,
                    1..=256 => {
                        let cycles_run = self.fetch_bg_tile_row();
                        self.cycles += cycles_run
                    }
                    257..=320 => todo!(),
                }
            }
        }
    }
    // TODO In general, we aren't handling any of the tricky
    // edge cases mentioned on the Registers NESDev page
    pub fn write_ppu_ctrl(&mut self, data: u8) {
        let prev_nmi_out = self.ppuctrl.contains(PPUCTRL::GENERATE_NMI);
        self.ppuctrl.update(data);
        if !prev_nmi_out
            && self.ppuctrl.contains(PPUCTRL::GENERATE_NMI)
            && self.ppustatus.contains(PPUSTATUS::VBLANK_START)
        {
            self.nmi_pin = true
        }
    }
    pub fn write_ppumask(&mut self, data: u8) {
        self.ppumask.update(data)
    }
    pub fn read_ppustatus(&mut self) -> u8 {
        self.ppustatus.set(PPUSTATUS::VBLANK_START, false);
        self.internal_reg.w = false;
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
        self.ppuscroll.update(data, self.internal_reg.w);
        self.internal_reg.w = !self.internal_reg.w;
    }
    pub fn write_ppuaddr(&mut self, data: u8) {
        self.ppuaddr.update(data, self.internal_reg.w);
        self.internal_reg.w = !self.internal_reg.w;
    }
    pub fn increment_ppu_addr(&mut self) {
        self.ppuaddr
            .increment_by(self.ppuctrl.contains(PPUCTRL::VRAM_ADDR_INCR))
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
        (self.oamaddr.0..=255)
            .zip(bytes)
            .for_each(|(idx, byte)| self.oam[idx as usize] = *byte)
    }
}
