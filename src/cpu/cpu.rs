use crate::{
    bus::Bus,
    utils::{join_hi_low, msb},
};

// flag locations for processor status register
const CARRY_FLAG: u8 = 0x01;
const ZERO_FLAG: u8 = 0x02;
const INTERRUPT_DISABLE: u8 = 0x03;
const DECIMAL_MODE: u8 = 0x04;
const BREAK_CMD: u8 = 0x05;
const OVERFLOW_FLAG: u8 = 0x07;
const NEGATIVE_FLAG: u8 = 0x08;

// Interrupt handlers
const NON_MASKABLE_IH: u16 = 0xfffa;
const POWER_RESET_IH: u16 = 0xfffc;
const BRK_IH: u16 = 0xfffe;

const STACK_END: u16 = 0x1ff;

struct CPU {
    pc: u16,
    sp: u8,
    accum: u8,
    rx: u8,
    ry: u8,
    st: u8,
    bus: Bus,
}

impl CPU {
    // TODO consider timing? (e.g. how many cycles instruction each runs)
    // TODO what should this return -- some state?
    // TODO consider how mirroring will work. should this be handled by the bus or the cpu
    //   answered: handled by the bus for the time being
    fn exec_opcode(&mut self, opcode: u8) {
        match opcode {
            // ADC - Add with Carry
            0x69 => {
                let v = self.immediate();
                self.adc(v)
            }
            0x65 => {
                let v = self.zero_page();
                self.adc(v)
            }
            0x75 => {
                let v = self.zero_page_x();
                self.adc(v)
            }
            0x6d => {
                let v = self.absolute();
                self.adc(v)
            }
            0x7d => {
                let v = self.absolute_x();
                self.adc(v)
            }
            0x79 => {
                let v = self.absolute_y();
                self.adc(v)
            }
            0x61 => {
                let v = self.indirect_x();
                self.adc(v)
            }
            0x71 => {
                let v = self.indirect_y();
                self.adc(v)
            }
            // ********
            // And - Logical AND
            0x29 => {
                let v = self.immediate();
                self.and(v)
            }
            0x25 => {
                let v = self.zero_page();
                self.and(v)
            }
            0x35 => {
                let v = self.zero_page_x();
                self.and(v)
            }
            0x2d => {
                let v = self.absolute();
                self.and(v)
            }
            0x3d => {
                let v = self.absolute_x();
                self.and(v)
            }
            0x39 => {
                let v = self.absolute_y();
                self.and(v)
            }
            0x21 => {
                let v = self.indirect_x();
                self.and(v)
            }
            0x31 => {
                let v = self.indirect_y();
                self.and(v)
            }
            // ********
            // ASL - Arithmetic Shift Left
            // TODO left off here!
            0x0a => {
                if msb(self.accum) == 1 {
                    self.set_carry()
                } else {
                    self.clear_carry()
                }

                let next_accum = self.accum << 1;

                if msb(self.accum) == 1 {
                    self.set_neg()
                } else {
                    self.clear_neg()
                }

                self.accum = next_accum
            }
        }
    }

    fn adc(&mut self, v: u8) {
        let next_accum = self.accum as u16 + (v as u16) + (self.st & CARRY_FLAG) as u16;
        let wrapped_accum = next_accum as u8;
        // TODO double check
        if msb(self.accum ^ wrapped_accum) & (v ^ wrapped_accum) == 1 {
            self.set_overflow()
        } else {
            self.clear_overflow()
        }

        if next_accum > 0xff {
            self.set_carry()
        } else {
            self.clear_carry()
        }

        if next_accum == 0 {
            self.set_zero()
        } else {
            self.clear_zero()
        }

        if msb(wrapped_accum) == 1 {
            self.set_neg()
        } else {
            self.clear_neg()
        }

        self.accum = wrapped_accum
    }

    fn and(&mut self, v: u8) {
        let result = self.accum & v;

        if result == 0 {
            self.set_zero()
        } else {
            self.clear_zero()
        }
        if msb(result) == 1 {
            self.set_neg()
        } else {
            self.clear_neg()
        }

        self.accum = result
    }

    // Indexed adressing functions
    // ** These functions update the program counter **
    fn immediate(&mut self) -> u8 {
        let result = self.bus.read_memory(self.pc + 1);
        self.pc += 2;
        result
    }
    fn zero_page(&mut self) -> u8 {
        let addr = self.bus.read_memory(self.pc + 1) as u16;
        let result = self.bus.read_memory(addr);
        self.pc += 2;
        result
    }
    fn zero_page_x(&mut self) -> u8 {
        let arg = self.bus.read_memory(self.pc + 1);
        let addr = arg.wrapping_add(self.rx);
        let result = self.bus.read_memory(addr as u16);
        self.pc += 2;
        result
    }
    fn zero_page_y(&mut self) -> u8 {
        let arg = self.bus.read_memory(self.pc + 1);
        let addr = arg.wrapping_add(self.ry);
        let result = self.bus.read_memory(addr as u16);
        self.pc += 2;
        result
    }
    fn absolute(&mut self) -> u8 {
        let lo = self.bus.read_memory(self.pc + 1);
        let hi = self.bus.read_memory(self.pc + 2);
        let result = self.bus.read_memory(join_hi_low(lo, hi));
        self.pc += 3;
        result
    }
    fn absolute_x(&mut self) -> u8 {
        let lo = self.bus.read_memory(self.pc + 1);
        let hi = self.bus.read_memory(self.pc + 2);
        let addr = join_hi_low(lo, hi);
        let result = self.bus.read_memory(addr + (self.rx as u16));
        self.pc += 3;
        result
    }
    fn absolute_y(&mut self) -> u8 {
        let lo = self.bus.read_memory(self.pc + 1);
        let hi = self.bus.read_memory(self.pc + 2);
        let addr = join_hi_low(lo, hi);
        let result = self.bus.read_memory(addr + (self.ry as u16));
        self.pc += 3;
        result
    }
    fn indirect_x(&mut self) -> u8 {
        let arg = self.bus.read_memory(self.pc + 1);
        let lo = self.bus.read_memory(arg.wrapping_add(self.rx) as u16);
        let hi = self
            .bus
            .read_memory(arg.wrapping_add(self.rx.wrapping_add(1)) as u16);
        let addr = join_hi_low(lo, hi);
        let result = self.bus.read_memory(addr);
        self.pc += 2;
        result
    }
    fn indirect_y(&mut self) -> u8 {
        let arg = self.bus.read_memory(self.pc + 1);
        let lo = self.bus.read_memory(arg as u16);
        let hi = self.bus.read_memory(arg.wrapping_add(1) as u16);
        let addr = join_hi_low(lo, hi) + (self.ry as u16);
        let result = self.bus.read_memory(addr);
        self.pc += 2;
        result
    }
    // ********

    // Status flag setters
    fn set_carry(&mut self) {
        self.set_st(CARRY_FLAG - 1)
    }
    fn clear_carry(&mut self) {
        self.clear_st(CARRY_FLAG - 1)
    }
    fn set_zero(&mut self) {
        self.set_st(ZERO_FLAG - 1)
    }
    fn clear_zero(&mut self) {
        self.clear_st(ZERO_FLAG - 1)
    }
    fn set_overflow(&mut self) {
        self.set_st(OVERFLOW_FLAG - 1)
    }
    fn clear_overflow(&mut self) {
        self.clear_st(OVERFLOW_FLAG - 1)
    }
    fn set_neg(&mut self) {
        self.set_st(NEGATIVE_FLAG - 1)
    }
    fn clear_neg(&mut self) {
        self.clear_st(NEGATIVE_FLAG - 1)
    }

    fn set_st(&mut self, n: u8) {
        self.st |= 1 << n
    }
    fn clear_st(&mut self, n: u8) {
        self.st &= !(1 << n)
    }
    // ********
}
