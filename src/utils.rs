// Various utility functions to be used by all
pub fn msb(value: u8) -> u8 {
    (value & 0x80) >> 7
}
pub fn join_hi_low(lo: u8, hi: u8) -> u16 {
    (hi as u16) << 0x8 | (lo as u16)
}
