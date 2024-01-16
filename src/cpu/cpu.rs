use crate::{
    bus::Bus,
    utils::{as_lo_hi, get_bit, join_hi_low, msb},
};

// flag locations for processor status register
const CARRY_FLAG: u8 = 0x01;
const ZERO_FLAG: u8 = 0x02;
const INTERRUPT_DISABLE: u8 = 0x03;
const DECIMAL_MODE: u8 = 0x04;
const BRK_CMD: u8 = 0x05;
const OVERFLOW_FLAG: u8 = 0x07;
const NEGATIVE_FLAG: u8 = 0x08;

// Interrupt handlers
const NON_MASKABLE_IH: u16 = 0xfffa;
const POWER_RESET_IH: u16 = 0xfffc;
const BRK_IH: u16 = 0xfffe;

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
            0x69 => self.adc(self.immediate()),
            0x65 => self.adc(self.zero_page().0),
            0x75 => self.adc(self.zero_page_x().0),
            0x6d => self.adc(self.absolute().0),
            0x7d => self.adc(self.absolute_x().0),
            0x79 => self.adc(self.absolute_y()),
            0x61 => self.adc(self.indirect_x()),
            0x71 => self.adc(self.indirect_y()),
            // ********
            // And - Logical AND
            0x29 => self.and(self.immediate()),
            0x25 => self.and(self.zero_page().0),
            0x35 => self.and(self.zero_page_x().0),
            0x2d => self.and(self.absolute().0),
            0x3d => self.and(self.absolute_x().0),
            0x39 => self.and(self.absolute_y()),
            0x21 => self.and(self.indirect_x()),
            0x31 => self.and(self.indirect_y()),
            // ********
            // ASL - Arithmetic Shift Left
            0x0a => {
                self.accum = self.asl(self.accum);
                self.pc += 1
            }
            0x06 => {
                let (v, addr) = self.zero_page();
                let result = self.asl(v);
                self.bus.write_memory(addr, result)
            }
            0x16 => {
                let (v, addr) = self.zero_page_x();
                let result = self.asl(v);
                self.bus.write_memory(addr, result)
            }
            0x0E => {
                let (v, addr) = self.absolute();
                let result = self.asl(v);
                self.bus.write_memory(addr, result)
            }
            0x1E => {
                let (v, addr) = self.absolute_x();
                let result = self.asl(v);
                self.bus.write_memory(addr, result)
            }
            // BCC - Branch if Carry Clear
            0x90 => {
                let arg = self.bus.read_memory(self.pc + 1);
                if self.get_st(CARRY_FLAG - 1) == 0 {
                    self.pc += arg as u16;
                }
                self.pc += 2;
            }
            // ********
            // BCS - Branch if Carry Set
            0xb0 => {
                let arg = self.bus.read_memory(self.pc + 1);
                if self.get_st(CARRY_FLAG - 1) == 1 {
                    self.pc += arg as u16;
                }
                self.pc += 2;
            }
            // ********
            // BEQ - Branch if Equal
            0xf0 => {
                let arg = self.bus.read_memory(self.pc + 1);
                if self.get_st(ZERO_FLAG - 1) == 1 {
                    self.pc += arg as u16;
                }
                self.pc += 2;
            }
            // ********
            // BIT - Bit Test
            0x24 => {
                let (v, _) = self.zero_page();
                self.bit(v)
            }
            0x2c => {
                let (v, _) = self.absolute();
                self.bit(v)
            }
            // ********
            // BMI - Branch if Minus
            0x30 => {
                let arg = self.bus.read_memory(self.pc + 1);
                if self.get_st(NEGATIVE_FLAG - 1) == 1 {
                    self.pc += arg as u16;
                }
                self.pc += 2
            }
            // ********
            // BNE - Branch if Not Equal
            0xd0 => {
                let arg = self.bus.read_memory(self.pc + 1);
                if self.get_st(ZERO_FLAG - 1) == 0 {
                    self.pc += arg as u16;
                }
                self.pc += 2
            }
            // ********
            // BPL - Branch if Positive
            0x10 => {
                let arg = self.bus.read_memory(self.pc + 1);
                if self.get_st(NEGATIVE_FLAG - 1) == 0 {
                    self.pc += arg as u16;
                }
                self.pc += 2
            }
            // ********
            // BRK - Force Interrupt
            0x00 => {
                self.brk();
                self.pc += 1
            }
            // BVC - Branch if Overflow Clear
            0x50 => {
                let arg = self.bus.read_memory(self.pc + 1);
                if self.get_st(OVERFLOW_FLAG - 1) == 0 {
                    self.pc += arg as u16;
                }
                self.pc += 2
            }
            // ********
            // BVS - Branch if Overflow Set
            0x70 => {
                let arg = self.bus.read_memory(self.pc + 1);
                if self.get_st(OVERFLOW_FLAG - 1) == 1 {
                    self.pc += arg as u16;
                }
                self.pc += 2
            }
            // ********
            // CLC - Clear Carry Flag
            0x18 => {
                self.clear_carry();
                self.pc += 2
            }
            // ********
            // CLD - Clear Decimal Mode
            0xd8 => {
                self.clear_decmimal();
                self.pc += 2
            }
            // ********
            // CLI - Clear Interrupt Disable
            0x58 => {
                self.clear_interrupt_disable();
                self.pc += 2
            }
            // ********
            // CLV - Clear Overflow Flag
            0xb8 => {
                self.clear_overflow();
                self.pc += 2
            }
            // ********
            // CMP - Compare
            0xc9 => self.cmp(self.immediate()),
            0xc5 => self.cmp(self.zero_page().0),
            0xd5 => self.cmp(self.zero_page_x().0),
            0xcd => self.cmp(self.absolute().0),
            0xdd => self.cmp(self.absolute_x().0),
            0xd9 => self.cmp(self.indirect_x()),
            0xd1 => self.cmp(self.indirect_y()),
            // ********
            // CPX - Compare X Register
            0xe0 => self.cpx(self.immediate()),
            0xe4 => self.cpx(self.zero_page().0),
            0xec => self.cpx(self.absolute().0),
            // ********
            // CPY - Compare Y Register
            0xc0 => self.cpy(self.immediate()),
            0xc4 => self.cpy(self.zero_page().0),
            0xcc => self.cpy(self.absolute().0),
            // ********
            // DEC - Decrement Memory
            0xc6 => {
                let (arg, addr) = self.zero_page();
                let result = self.dec(arg);
                self.bus.write_memory(addr, result)
            }
            0xd6 => {
                let (arg, addr) = self.zero_page_x();
                let result = self.dec(arg);
                self.bus.write_memory(addr, result)
            }
            0xce => {
                let (arg, addr) = self.absolute();
                let result = self.dec(arg);
                self.bus.write_memory(addr, result)
            }
            0xde => {
                let (arg, addr) = self.absolute_x();
                let result = self.dec(arg);
                self.bus.write_memory(addr, result)
            }
            // ********
            // DEX - Decrement X Register
            0xca => {
                let result = self.rx.wrapping_sub(1);
                self.cond_set_zero(result == 0);
                self.cond_set_neg(msb(result) == 1);
                self.rx = result
            }
            // ********
            // DEY - Decrement Y Register
            0x88 => {
                let result = self.ry.wrapping_sub(1);
                self.cond_set_zero(result == 0);
                self.cond_set_neg(msb(result) == 1);
                self.ry = result
            }
            // ********
            // EOR - Exclusive OR
            0x49 => self.eor(self.immediate()),
            0x45 => self.eor(self.zero_page().0),
            0x55 => self.eor(self.zero_page_x().0),
            0x4d => self.eor(self.absolute().0),
            0x5d => self.eor(self.absolute_x().0),
            0x59 => self.eor(self.absolute_y()),
            0x41 => self.eor(self.indirect_x()),
            0x51 => self.eor(self.indirect_y()),
            // ********
            // INC - Increment Memory
            0xe6 => {
                let (arg, addr) = self.zero_page();
                let result = self.inc(arg);
                self.bus.write_memory(addr, result)
            }
            0xf6 => {
                let (arg, addr) = self.zero_page_x();
                let result = self.inc(arg);
                self.bus.write_memory(addr, result)
            }
            0xee => {
                let (arg, addr) = self.absolute();
                let result = self.inc(arg);
                self.bus.write_memory(addr, result)
            }
            0xfe => {
                let (arg, addr) = self.absolute_x();
                let result = self.inc(arg);
                self.bus.write_memory(addr, result)
            }
            // ********
            // INX - Increment X Register
            0xe8 => {
                let result = self.rx.wrapping_add(1);
                self.cond_set_zero(result == 0);
                self.cond_set_neg(msb(result) == 1);
                self.rx = result
            }
            // ********
            // INY - Increment Y Register
            0xc8 => {
                let result = self.ry.wrapping_add(1);
                self.cond_set_zero(result == 0);
                self.cond_set_neg(msb(result) == 1);
                self.ry = result
            }
            // ********
            // JMP - Jump
            0x4c => {
                let lo = self.bus.read_memory(self.pc + 1);
                let hi = self.bus.read_memory(self.pc + 2);
                let addr = join_hi_low(lo, hi);
                self.pc = addr
            }
            0x6c => {
                /* Indirect JMP */
                let lo_ind = self.bus.read_memory(self.pc + 1);
                let hi_ind = self.bus.read_memory(self.pc + 2);
                let addr_ind = join_hi_low(lo_ind, hi_ind);
                let lo = self.bus.read_memory(addr_ind);
                let hi = self.bus.read_memory(addr_ind + 1);
                self.pc = join_hi_low(lo, hi)
            }
            // ********
            // JSR - Jump to Subroutine
            0x20 => {
                let lo = self.bus.read_memory(self.pc + 1);
                let hi = self.bus.read_memory(self.pc + 2);
                let addr = join_hi_low(lo, hi);
                let (lo_ret, hi_ret) = as_lo_hi(addr - 1);
                self.stack_push(hi_ret);
                self.stack_push(lo_ret);
                self.pc = addr
            }
            // ********
            // LDA - Load Accumulator
            0xa9 => self.lda(self.immediate()),
            0xa5 => self.lda(self.zero_page().0),
            0xb5 => self.lda(self.zero_page_x().0),
            0xad => self.lda(self.absolute().0),
            0xbd => self.lda(self.absolute_x().0),
            0xb9 => self.lda(self.absolute_y()),
            0xa1 => self.lda(self.indirect_x()),
            0xb1 => self.lda(self.indirect_y()),
            // ********
            // LDX - Load X Register
            0xa2 => self.ldx(self.immediate()),
            0xa6 => self.ldx(self.zero_page().0),
            0xb6 => self.ldx(self.zero_page_y()),
            0xae => self.ldx(self.absolute().0),
            0xbe => self.ldx(self.absolute_y()),
            // ********
            // LDY - Load Y Register
            0xa0 => self.ldy(self.immediate()),
            0xa4 => self.ldy(self.zero_page().0),
            0xb4 => self.ldy(self.zero_page_x().0),
            0xac => self.ldy(self.absolute().0),
            0xbc => self.ldy(self.absolute_x().0),
            // ********
            // LSR - Logical Shift Right
            0x4a => {
                self.accum = self.lsr(self.accum);
                self.pc += 1
            }
            0x46 => {
                let (v, addr) = self.zero_page();
                let result = self.lsr(v);
                self.bus.write_memory(addr, result);
            }
            0x56 => {
                let (v, addr) = self.zero_page_x();
                let result = self.lsr(v);
                self.bus.write_memory(addr, result);
            }
            0x4e => {
                let (v, addr) = self.absolute();
                let result = self.lsr(v);
                self.bus.write_memory(addr, result);
            }
            0x5e => {
                let (v, addr) = self.absolute_x();
                let result = self.lsr(v);
                self.bus.write_memory(addr, result);
            }
            // ********
            // NOP - No Operation
            0xea => self.pc += 1,
            // ********
            // ORA - Logical Inclusive OR
            0x09 => self.ora(self.immediate()),
            0x05 => self.ora(self.zero_page().0),
            0x15 => self.ora(self.zero_page_x().0),
            0x0d => self.ora(self.absolute().0),
            0x1d => self.ora(self.absolute_x().0),
            0x19 => self.ora(self.absolute_y()),
            0x01 => self.ora(self.indirect_x()),
            0x11 => self.ora(self.indirect_y()),
            // ********
            // PHA - Push Accumulator
            0x48 => {
                self.stack_push(self.accum);
                self.pc += 1
            }
            // ********
            // PHP - Push Processor Status
            0x08 => {
                self.stack_push(self.st);
                self.pc += 1
            }
            // ********
            // PLA - Pull Accumulator
            0x68 => {
                let next_accum = self.stack_pop();
                self.cond_set_zero(next_accum == 0);
                self.cond_set_neg(msb(next_accum) == 1);
                self.accum = next_accum;
                self.pc += 1
            }
            // ********
            // PLP - Pull Processor Status
            // TODO the processor flag status changes here are not 100% clear to me
            0x28 => {
                let next_st = self.stack_pop();
                self.st = next_st;
                self.pc += 1
            } 
            // ********
            // ROL - Rotate Left
            // TODO left off here...
            // ********
        }
    }

    fn adc(&mut self, v: u8) {
        let next_accum = self.accum as u16 + (v as u16) + (self.st & CARRY_FLAG) as u16;
        let wrapped_accum = next_accum as u8;
        // TODO double check
        self.cond_set_overflow(msb(self.accum ^ wrapped_accum) & (v ^ wrapped_accum) == 1);

        self.cond_set_carry(next_accum > 0xff);

        self.cond_set_zero(next_accum == 0);

        self.cond_set_neg(msb(wrapped_accum) == 1);

        self.accum = wrapped_accum
    }

    fn and(&mut self, v: u8) {
        let result = self.accum & v;

        self.cond_set_zero(result == 0);

        self.cond_set_neg(msb(result) == 1);

        self.accum = result
    }

    fn asl(&mut self, v: u8) -> u8 {
        self.cond_set_carry(msb(v) == 1);

        let next_v = v << 1;
        self.cond_set_neg(msb(next_v) == 1);

        next_v
    }

    fn bit(&mut self, v: u8) {
        self.cond_set_zero(self.accum & v == 0);

        self.set_st_to(OVERFLOW_FLAG - 1, get_bit(&v, 6));
        self.set_st_to(NEGATIVE_FLAG - 1, get_bit(&v, 7))
    }

    fn brk(&mut self) {
        let low_pc = (self.pc & 0xff) as u8;
        let hi_pc = ((self.pc >> 8) & 0xff) as u8;

        // push current pc and status flag to stack (in that orer)
        self.stack_push(hi_pc);
        self.stack_push(low_pc);
        self.stack_push(self.st);

        // load IRQ interrupt vector
        let low_addr = self.bus.read_memory(BRK_IH);
        let hi_addr = self.bus.read_memory(BRK_IH + 1);

        let ih_addr = join_hi_low(low_addr, hi_addr);
        self.pc = ih_addr;

        self.set_brk()
    }

    fn cmp(&mut self, v: u8) {
        self.cond_set_carry(self.accum >= v);
        self.cond_set_zero(self.accum == v);
        let result = self.accum - v;
        self.cond_set_neg(msb(result) == 1);
    }
    fn cpx(&mut self, v: u8) {
        self.cond_set_carry(self.rx >= v);
        self.cond_set_zero(self.rx == v);
        let result = self.rx - v;
        self.cond_set_neg(msb(result) == 1);
    }
    fn cpy(&mut self, v: u8) {
        self.cond_set_carry(self.ry >= v);
        self.cond_set_zero(self.ry == v);
        let result = self.ry - v;
        self.cond_set_neg(msb(result) == 1);
    }
    fn dec(&mut self, v: u8) -> u8 {
        let result = v.wrapping_sub(1);
        self.cond_set_zero(result == 0);
        self.cond_set_neg(msb(result) == 1);

        result
    }
    fn eor(&mut self, v: u8) {
        let result = self.accum ^ v;
        self.cond_set_zero(result == 0);
        self.cond_set_neg(msb(result) == 1);
        self.accum = result
    }
    fn inc(&mut self, v: u8) -> u8 {
        let result = v.wrapping_add(1);
        self.cond_set_zero(result == 0);
        self.cond_set_neg(msb(result) == 1);

        result
    }
    fn lda(&mut self, v: u8) {
        self.cond_set_zero(v == 0);
        self.cond_set_neg(msb(v) == 1);

        self.accum = v;
    }
    fn ldx(&mut self, v: u8) {
        self.cond_set_zero(v == 0);
        self.cond_set_neg(msb(v) == 1);

        self.rx = v;
    }
    fn ldy(&mut self, v: u8) {
        self.cond_set_zero(v == 0);
        self.cond_set_neg(msb(v) == 1);

        self.ry = v;
    }
    fn lsr(&mut self, v: u8) -> u8 {
        self.set_st_to(CARRY_FLAG - 1, v & 0x01);
        let result = v.wrapping_shr(1);
        self.cond_set_zero(result == 0);
        self.cond_set_neg(msb(result) == 1);
        result
    }
    fn ora(&mut self, v: u8) {
        let result = self.accum | v;
        self.cond_set_zero(result == 0);
        self.cond_set_neg(msb(result) == 1);
        self.accum = result
    }

    // Indexed adressing functions
    // ** These functions update the program counter **

    // functions to get the underlying data value for the given adressing mode
    fn immediate(&mut self) -> u8 {
        let result = self.bus.read_memory(self.pc + 1);
        self.pc += 2;
        result
    }
    fn zero_page(&mut self) -> (u8, u16) {
        let addr = self.bus.read_memory(self.pc + 1) as u16;
        let result = self.bus.read_memory(addr);
        self.pc += 2;
        (result, addr)
    }
    fn zero_page_x(&mut self) -> (u8, u16) {
        let arg = self.bus.read_memory(self.pc + 1);
        let addr = arg.wrapping_add(self.rx);
        let result = self.bus.read_memory(addr as u16);
        self.pc += 2;
        (result, addr as u16)
    }
    fn zero_page_y(&mut self) -> u8 {
        let arg = self.bus.read_memory(self.pc + 1);
        let addr = arg.wrapping_add(self.ry);
        let result = self.bus.read_memory(addr as u16);
        self.pc += 2;
        result
    }
    fn absolute(&mut self) -> (u8, u16) {
        let lo = self.bus.read_memory(self.pc + 1);
        let hi = self.bus.read_memory(self.pc + 2);
        let addr = join_hi_low(lo, hi);
        let result = self.bus.read_memory(addr);
        self.pc += 3;
        (result, addr)
    }
    fn absolute_x(&mut self) -> (u8, u16) {
        let lo = self.bus.read_memory(self.pc + 1);
        let hi = self.bus.read_memory(self.pc + 2);
        let addr = join_hi_low(lo, hi);
        let result = self.bus.read_memory(addr + (self.rx as u16));
        self.pc += 3;
        (result, addr)
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
    fn cond_set_carry(&mut self, cond: bool) {
        if cond {
            self.set_carry()
        } else {
            self.clear_carry()
        }
    }
    fn set_carry(&mut self) {
        self.set_st(CARRY_FLAG - 1)
    }
    fn clear_carry(&mut self) {
        self.clear_st(CARRY_FLAG - 1)
    }
    fn cond_set_zero(&mut self, cond: bool) {
        if cond {
            self.set_zero()
        } else {
            self.clear_zero()
        }
    }
    fn set_zero(&mut self) {
        self.set_st(ZERO_FLAG - 1)
    }
    fn clear_zero(&mut self) {
        self.clear_st(ZERO_FLAG - 1)
    }
    fn cond_set_overflow(&mut self, cond: bool) {
        if cond {
            self.set_overflow()
        } else {
            self.clear_overflow()
        }
    }
    fn set_overflow(&mut self) {
        self.set_st(OVERFLOW_FLAG - 1)
    }
    fn clear_overflow(&mut self) {
        self.clear_st(OVERFLOW_FLAG - 1)
    }
    fn cond_set_neg(&mut self, cond: bool) {
        if cond {
            self.set_neg()
        } else {
            self.clear_neg()
        }
    }
    fn set_neg(&mut self) {
        self.set_st(NEGATIVE_FLAG - 1)
    }
    fn clear_neg(&mut self) {
        self.clear_st(NEGATIVE_FLAG - 1)
    }

    fn set_brk(&mut self) {
        self.set_st(BRK_CMD - 1)
    }
    fn clear_brk(&mut self) {
        self.clear_st(BRK_CMD - 1)
    }
    fn set_decimal(&mut self) {
        self.set_st(DECIMAL_MODE - 1)
    }
    fn clear_decmimal(&mut self) {
        self.clear_st(DECIMAL_MODE - 1)
    }
    fn set_interrupt_disable(&mut self) {
        self.set_st(INTERRUPT_DISABLE - 1)
    }
    fn clear_interrupt_disable(&mut self) {
        self.clear_st(INTERRUPT_DISABLE - 1)
    }

    fn get_st(&self, n: u8) -> u8 {
        get_bit(&self.st, n)
    }
    fn set_st(&mut self, n: u8) {
        self.set_st_to(n, 1)
    }
    /**
     * Set bit n (0-indexed) to value v. Value v is &'d with 0x01
     * to ensure v is either 0 or 1.
     */
    fn set_st_to(&mut self, n: u8, v: u8) {
        self.st |= (v & 0x01) << n
    }
    fn clear_st(&mut self, n: u8) {
        self.st &= !(1 << n)
    }
    // ********

    /* Stack Functions
    self.sp points to the value currently on "top" of the stack.
    If an address (e.g. a u16 value) is "pushed" onto the stack, we should
    remain consistent in that we push the hi byte first, _then_ the low byte
    */
    fn stack_push(&mut self, byte: u8) {
        self.sp = self.sp.wrapping_sub(1);
        let addr = 0x100 + self.sp as u16;
        self.bus.write_memory(addr, byte)
    }
    fn stack_pop(&mut self) -> u8 {
        let addr = 0x100 + self.sp as u16;
        let result = self.bus.read_memory(addr);
        self.sp = self.sp.wrapping_add(1);
        result
    }
    // ********
}
