// Various utility functions to be used by all
pub fn msb(value: u8) -> u8 {
    (value & 0x80) >> 7
}
pub fn join_hi_low(lo: u8, hi: u8) -> u16 {
    (hi as u16) << 0x8 | (lo as u16)
}

/**
 * Breaks an addr into (lo, hi) bytes.
 */
pub fn as_lo_hi(addr: u16) -> (u8, u8) {
    ((addr & 0xff) as u8, ((addr >> 0x8) & 0xff) as u8)
}

pub fn get_bit(byte: &u8, n: u8) -> u8 {
    byte >> n & 0x01
}
