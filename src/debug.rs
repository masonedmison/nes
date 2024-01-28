#[derive(Default, Debug, PartialEq)]
pub struct CpuState {
    pub addr: u16,
    pub opcode: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub sp: u8,
    pub cycles: u64,
}

impl CpuState {
    pub fn render(&self) -> String {
        format!(
            "Address={:#x}\tOpcode={:#x}\tA:{:#x}\tX:{:#x}\tY:{:#x}\tP:{:#x}\tSP:{:#x}\tCycles:{}",
            self.addr, self.opcode, self.a, self.x, self.y, self.p, self.sp, self.cycles
        )
    }

    // sets status such that it's stored in line with what nestest.log expects
    pub fn set_status(&mut self, data: u8) {
        // bit 5 should always be 1 for p register
        let p = data | (1 << 5);
        // always print BRK flag is 0, this shouldn't have any noticable effect
        self.p = p & !(1 << 4)
    }
}
