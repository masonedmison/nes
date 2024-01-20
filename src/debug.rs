use std::fmt::format;

#[derive(Default, Debug)]
pub struct CpuState {
  pub addr: u16,
  pub opcode: u8,
  pub read_memory: String,
  pub a: u8,
  pub x: u8,
  pub y: u8,
  pub p: u8,
  pub sp: u8,
  pub cycles: u32
}

impl CpuState {
  pub fn render(&self) -> String {
    // bit 5 should always be 1 for p register
    let mut p = self.p | (1 << 5);
    // always print BRK flag is 0, this shouldn't have any noticable effect
    p &= !(1 << 4);

    format!(
      "Address={:#x}\tOpcode={:#x}\tA:{:#x}\tX:{:#x}\tY:{:#x}\tP:{:#x}\tSP:{:#x}",
      self.addr,
      self.opcode,
      self.a,
      self.x,
      self.y,
      p,
      self.sp
    )
  }
}
