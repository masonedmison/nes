use bitflags::bitflags;

// Memory-mapped registers read from and written to by CPU
bitflags! {
  // 0x2000 - Write
  pub struct PPUCTRL: u8 {
    const BASE_NAMETABLE_0 = 0x01;
    const BASE_NAMETABLE_1 = 0b00000010;
    const VRAM_ADDR_INCR = 0b00000100;
    const SPRITE_TABLE_ADDR = 0b00001000;
    const BACKGROUND_PATTERN_TABLE = 0b00010000;
    const SPRITE_SIZE = 0b00100000;
    const PPU_MASTER_SLAVE = 0b01000000;
    const GENERATE_NMI = 0b10000000;
  }
}

impl PPUCTRL {
  pub fn new() -> Self {
    PPUCTRL { bits: 0 }
  }
  pub fn update(&mut self, value: u8) {
    self.bits = value
  }
}

bitflags! {
  // 0x2001 - Write
  pub struct PPUMASK: u8 {
    const GRAYSCALE = 0x01;
    const SHOW_BACKGROUND_LEFTMOST = 0b00000010;
    const SHOW_SPRITES_LEFTMOST = 0b00000100;
    const SHOW_BACKGROUND = 0b00001000;
    const SHOW_SPRITE = 0b00010000;
    const EMPH_RED = 0b00001000;
    const EMPH_GREEN = 0b01000000;
    const EMPH_BLUE = 0b10000000;
  }
}
impl PPUMASK {
  pub fn new() -> Self {
    PPUMASK { bits: 0 }
  }
  pub fn update(&mut self, value: u8) {
    self.bits = value
  }
}

bitflags! {
  // 0x2002
  pub struct PPUSTATUS: u8 {
    const SPRITE_OVERFLOW = 0b00100000;
    const SPRITE_0_HIT = 0b01000000;
    const VBLANK_START = 0b10000000;
  }
}
impl PPUSTATUS {
  pub fn new() -> Self {
    PPUSTATUS { bits: 0 }
  }
}

// 0x2003
pub struct OAMADDR(pub u8);

// 0x2004
pub struct OAMDATA(pub u8);

// 2005
pub struct PPUSCROLL {
  // (x scroll, y scroll)
  value: (u8, u8),
}
impl PPUSCROLL {
  pub fn new () -> PPUSCROLL {
    PPUSCROLL { value: (0, 0) }
  }
  pub fn update(&mut self, value: u8, w: bool) {
    if w { self.value.1 = value}
    else { self.value.0 = value }
  }
}

// 2006
pub struct PPUADDR {
  // (msb, lsb)
  value: (u8, u8),
}
impl PPUADDR {
  pub fn new () -> PPUADDR {
    // (hi, low)
    PPUADDR { value: (0, 0) }
  }
  /**
   * Get mirrors the underlying hi and lo bytes
   * to 0x00..=0x3fff if the combined bytes exceeds 14 bits
   */
  pub fn get(&self) -> u16 {
    let combined = (self.value.0 << 8) as u16 | (self.value.1) as u16;
    combined & 0x3fff
  }
  fn set(&mut self, data: u16) {
    let hi = data >> 8;
    let lo = data & 0xff;
    self.value = (hi as u8, lo as u8)
  }
  pub fn update(&mut self, data: u8, w: bool) {
    if w { self.value.1 = data}
    else { self.value.0 = data }
  }
  pub fn increment_by(&mut self, bit: bool) {
    let incr_by = (if bit { 32 } else { 1 }) as u16;
    self.set(self.get() + incr_by)
  }
}

// 2007
pub struct PPUDATA(pub u8);
// ********
