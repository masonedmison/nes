use crate::{
    bus::Bus,
    cartridge::Cartridge,
    debug::CpuState,
    utils::{as_lo_hi, get_bit, join_hi_low, msb},
};

// flag locations (1-indexed) for processor status register
const CARRY_FLAG: u8 = 0x01;
const ZERO_FLAG: u8 = 0x02;
const INTERRUPT_DISABLE: u8 = 0x03;
const DECIMAL_MODE: u8 = 0x04;
/**
 *  Instruction 	B 	Side effects after pushing
 *  /IRQ 	0 	I is set to 1
 *  /NMI 	0 	I is set to 1
 *   BRK 	1 	I is set to 1
 *   PHP 	1 	None
 */
const BRK_CMD: u8 = 0x05;
const OVERFLOW_FLAG: u8 = 0x07;
const NEGATIVE_FLAG: u8 = 0x08;

// Interrupt handlers
const NON_MASKABLE_IH: u16 = 0xfffa;
const POWER_RESET_IH: u16 = 0xfffc;
const BRK_IH: u16 = 0xfffe;

pub struct CPU {
    pc: u16,
    sp: u8,
    accum: u8,
    rx: u8,
    ry: u8,
    st: u8,
    bus: Bus,
}

impl CPU {
    pub fn new() -> CPU {
        let bus = Bus::new();
        // TODO setting this to match starting state of nestest.nes
        CPU {
            pc: 0xC000,
            sp: 0xfd,
            accum: 0,
            rx: 0,
            ry: 0,
            st: 0x24,
            bus,
        }
    }

    fn reset(&mut self) {
        self.rx = 0;
        self.ry = 0;
        self.st = 0;

        self.pc = join_hi_low(
            self.bus.read_memory(POWER_RESET_IH),
            self.bus.read_memory(POWER_RESET_IH + 1),
        )
    }
    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.reset();
        self.bus.load_rom(cartridge.bytes)
    }

    pub fn run_debug(&mut self) {
        loop {
            let opcode = self.bus.read_memory(self.pc);
            self.debug_exec(opcode)
        }
    }
    fn debug_exec(&mut self, opcode: u8) {
        let mut state = CpuState::default();
        state.opcode = opcode;
        state.addr = self.pc;
        state.a = self.accum;
        state.x = self.rx;
        state.y = self.ry;
        state.sp = self.sp;
        state.p = self.st;

        println!("{}", state.render());

        self.exec_opcode(opcode);
    }
    // TODO consider timing? (e.g. how many cycles instruction each runs)
    fn exec_opcode(&mut self, opcode: u8) {
        match opcode {
            // ADC - Add with Carry
            0x69 => {
                let v = self.immediate();
                self.adc(v)
            }
            0x65 => {
                let zero_page = self.zero_page();
                self.adc(zero_page.0)
            }
            0x75 => {
                let zero_page_x = self.zero_page_x();
                self.adc(zero_page_x.0)
            }
            0x6d => {
                let absolute = self.absolute();
                self.adc(absolute.0)
            }
            0x7d => {
                let absolute_x = self.absolute_x();
                self.adc(absolute_x.0)
            }
            0x79 => {
                let (v, _) = self.absolute_y();
                self.adc(v)
            }
            0x61 => {
                let (v, _) = self.indirect_x();
                self.adc(v)
            }
            0x71 => {
                let (v, _) = self.indirect_y();
                self.adc(v)
            }
            // ********
            // And - Logical AND
            0x29 => {
                let v = self.immediate();
                self.and(v)
            }
            0x25 => {
                let zero_page = self.zero_page();
                self.and(zero_page.0)
            }
            0x35 => {
                let zero_page_x = self.zero_page_x();
                self.and(zero_page_x.0)
            }
            0x2d => {
                let absolute = self.absolute();
                self.and(absolute.0)
            }
            0x3d => {
                let absolute_x = self.absolute_x();
                self.and(absolute_x.0)
            }
            0x39 => {
                let (v, _) = self.absolute_y();
                self.and(v)
            }
            0x21 => {
                let (v, _) = self.indirect_x();
                self.and(v)
            }
            0x31 => {
                let (v, _) = self.indirect_y();
                self.and(v)
            }
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
                self.pc += 1
            }
            // ********
            // CLD - Clear Decimal Mode
            0xd8 => {
                self.clear_decmimal();
                self.pc += 1
            }
            // ********
            // CLI - Clear Interrupt Disable
            0x58 => {
                self.clear_interrupt_disable();
                self.pc += 1
            }
            // ********
            // CLV - Clear Overflow Flag
            0xb8 => {
                self.clear_overflow();
                self.pc += 1
            }
            // ********
            // CMP - Compare
            0xc9 => {
                let v = self.immediate();
                self.cmp(v)
            }
            0xc5 => {
                let zero_page = self.zero_page();
                self.cmp(zero_page.0)
            }
            0xd5 => {
                let zero_page_x = self.zero_page_x();
                self.cmp(zero_page_x.0)
            }
            0xcd => {
                let absolute = self.absolute();
                self.cmp(absolute.0)
            }
            0xdd => {
                let absolute_x = self.absolute_x();
                self.cmp(absolute_x.0)
            }
            0xd9 => {
                let (v, _) = self.absolute_y();
                self.cmp(v)
            }
            0xc1 => {
                let (v, _) = self.indirect_x();
                self.cmp(v)
            }
            0xd1 => {
                let (v, _) = self.indirect_y();
                self.cmp(v)
            }
            // ********
            // CPX - Compare X Register
            0xe0 => {
                let v = self.immediate();
                self.cpx(v)
            }
            0xe4 => {
                let zero_page = self.zero_page();
                self.cpx(zero_page.0)
            }
            0xec => {
                let absolute = self.absolute();
                self.cpx(absolute.0)
            }
            // ********
            // CPY - Compare Y Register
            0xc0 => {
                let v = self.immediate();
                self.cpy(v)
            }
            0xc4 => {
                let zero_page = self.zero_page();
                self.cpy(zero_page.0)
            }
            0xcc => {
                let absolute = self.absolute();
                self.cpy(absolute.0)
            }
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
                self.rx = result;
                self.pc += 1
            }
            // ********
            // DEY - Decrement Y Register
            0x88 => {
                let result = self.ry.wrapping_sub(1);
                self.cond_set_zero(result == 0);
                self.cond_set_neg(msb(result) == 1);
                self.ry = result;
                self.pc += 1
            }
            // ********
            // EOR - Exclusive OR
            0x49 => {
                let v = self.immediate();
                self.eor(v)
            }
            0x45 => {
                let zero_page = self.zero_page();
                self.eor(zero_page.0)
            }
            0x55 => {
                let zero_page_x = self.zero_page_x();
                self.eor(zero_page_x.0)
            }
            0x4d => {
                let absolute = self.absolute();
                self.eor(absolute.0)
            }
            0x5d => {
                let absolute_x = self.absolute_x();
                self.eor(absolute_x.0)
            }
            0x59 => {
                let (v, _) = self.absolute_y();
                self.eor(v)
            }
            0x41 => {
                let (v, _) = self.indirect_x();
                self.eor(v)
            }
            0x51 => {
                let (v, _) = self.indirect_y();
                self.eor(v)
            }
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
                self.rx = result;
                self.pc += 1
            }
            // ********
            // INY - Increment Y Register
            0xc8 => {
                let result = self.ry.wrapping_add(1);
                self.cond_set_zero(result == 0);
                self.cond_set_neg(msb(result) == 1);
                self.ry = result;
                self.pc += 1
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
                /*
                Indirect JMP
                NB:
                   An original 6502 has does not correctly fetch the target address if the indirect vector
                   falls on a page boundary (e.g. $xxFF where xx is any value from $00 to $FF). In this case
                   fetches the LSB from $xxFF as expected but takes the MSB from $xx00. This is fixed in
                   some later chips like the 65SC02 so for compatibility always ensure the indirect
                    vector is not at the end of the page.
                */
                let lo_ind = self.bus.read_memory(self.pc + 1);
                let hi_ind = self.bus.read_memory(self.pc + 2);
                let page_addr = (hi_ind as u16) << 8;
                let lo = self.bus.read_memory(page_addr | lo_ind as u16);
                let hi = self
                    .bus
                    .read_memory(page_addr | (lo_ind.wrapping_add(1)) as u16);
                self.pc = join_hi_low(lo, hi)
            }
            // ********
            // JSR - Jump to Subroutine
            0x20 => {
                let (lo_ret, hi_ret) = as_lo_hi(self.pc + 2);
                self.stack_push(hi_ret);
                self.stack_push(lo_ret);

                let lo = self.bus.read_memory(self.pc + 1);
                let hi = self.bus.read_memory(self.pc + 2);
                let addr = join_hi_low(lo, hi);
                self.pc = addr
            }
            // ********
            // LDA - Load Accumulator
            0xa9 => {
                let v = self.immediate();
                self.lda(v)
            }
            0xa5 => {
                let zero_page = self.zero_page();
                self.lda(zero_page.0)
            }
            0xb5 => {
                let zero_page_x = self.zero_page_x();
                self.lda(zero_page_x.0)
            }
            0xad => {
                let absolute = self.absolute();
                self.lda(absolute.0)
            }
            0xbd => {
                let absolute_x = self.absolute_x();
                self.lda(absolute_x.0)
            }
            0xb9 => {
                let (v, _) = self.absolute_y();
                self.lda(v)
            }
            0xa1 => {
                let (v, _) = self.indirect_x();
                self.lda(v)
            }
            0xb1 => {
                let (v, _) = self.indirect_y();
                self.lda(v)
            }
            // ********
            // LDX - Load X Register
            0xa2 => {
                let v = self.immediate();
                self.ldx(v)
            }
            0xa6 => {
                let zero_page = self.zero_page();
                self.ldx(zero_page.0)
            }
            0xb6 => {
                let (v, _) = self.zero_page_y();
                self.ldx(v)
            }
            0xae => {
                let absolute = self.absolute();
                self.ldx(absolute.0)
            }
            0xbe => {
                let (v, _) = self.absolute_y();
                self.ldx(v)
            }
            // ********
            // LDY - Load Y Register
            0xa0 => {
                let v = self.immediate();
                self.ldy(v)
            }
            0xa4 => {
                let zero_page = self.zero_page();
                self.ldy(zero_page.0)
            }
            0xb4 => {
                let zero_page_x = self.zero_page_x();
                self.ldy(zero_page_x.0)
            }
            0xac => {
                let absolute = self.absolute();
                self.ldy(absolute.0)
            }
            0xbc => {
                let absolute_x = self.absolute_x();
                self.ldy(absolute_x.0)
            }
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
            0x09 => {
                let v = self.immediate();
                self.ora(v)
            }
            0x05 => {
                let zero_page = self.zero_page();
                self.ora(zero_page.0)
            }
            0x15 => {
                let zero_page_x = self.zero_page_x();
                self.ora(zero_page_x.0)
            }
            0x0d => {
                let absolute = self.absolute();
                self.ora(absolute.0)
            }
            0x1d => {
                let absolute_x = self.absolute_x();
                self.ora(absolute_x.0)
            }
            0x19 => {
                let (v, _) = self.absolute_y();
                self.ora(v)
            }
            0x01 => {
                let (v, _) = self.indirect_x();
                self.ora(v)
            }
            0x11 => {
                let (v, _) = self.indirect_y();
                self.ora(v)
            }
            // ********
            // PHA - Push Accumulator
            0x48 => {
                self.stack_push(self.accum);
                self.pc += 1
            }
            // ********
            // PHP - Push Processor Status
            0x08 => {
                self.clear_brk();
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
            0x28 => {
                let next_st = self.stack_pop();
                self.st = next_st;
                self.pc += 1
            }
            // ********
            // ROL - Rotate Left
            0x2a => {
                self.accum = self.rol(self.accum);
                self.pc += 1
            }
            0x26 => {
                let (v, addr) = self.zero_page();
                let result = self.rol(v);
                self.bus.write_memory(addr, result);
            }
            0x36 => {
                let (v, addr) = self.zero_page_x();
                let result = self.rol(v);
                self.bus.write_memory(addr, result);
            }
            0x2e => {
                let (v, addr) = self.absolute();
                let result = self.rol(v);
                self.bus.write_memory(addr, result);
            }
            0x3e => {
                let (v, addr) = self.absolute_x();
                let result = self.rol(v);
                self.bus.write_memory(addr, result);
            }
            // ********
            // ROR - Rotate Right
            0x6a => {
                self.accum = self.ror(self.accum);
                self.pc += 1
            }
            0x66 => {
                let (v, addr) = self.zero_page();
                let result = self.ror(v);
                self.bus.write_memory(addr, result);
            }
            0x76 => {
                let (v, addr) = self.zero_page_x();
                let result = self.ror(v);
                self.bus.write_memory(addr, result);
            }
            0x6e => {
                let (v, addr) = self.absolute();
                let result = self.ror(v);
                self.bus.write_memory(addr, result);
            }
            0x7e => {
                let (v, addr) = self.absolute_x();
                let result = self.ror(v);
                self.bus.write_memory(addr, result);
            }
            // ********
            // RTI - Return from Interrupt
            0x40 => {
                self.st = self.stack_pop();
                let lo = self.stack_pop();
                let hi = self.stack_pop();
                self.pc = join_hi_low(lo, hi);
            }
            // ********
            // RTS - Return from Subroutine
            0x60 => {
                let lo = self.stack_pop();
                let hi = self.stack_pop();
                self.pc = join_hi_low(lo, hi).wrapping_add(1)
            }
            // ********
            // SBC - Subtract with Carry
            0xe9 => {
                let v = self.immediate();
                self.sbc(v)
            }
            0xe5 => {
                let zero_page = self.zero_page();
                self.sbc(zero_page.0)
            }
            0xf5 => {
                let zero_page_x = self.zero_page_x();
                self.sbc(zero_page_x.0)
            }
            0xed => {
                let absolute = self.absolute();
                self.sbc(absolute.0)
            }
            0xfd => {
                let absolute_x = self.absolute_x();
                self.sbc(absolute_x.0)
            }
            0xf9 => {
                let (v, _) = self.absolute_y();
                self.sbc(v)
            }
            0xe1 => {
                let (v, _) = self.indirect_x();
                self.sbc(v)
            }
            0xf1 => {
                let (v, _) = self.indirect_y();
                self.sbc(v)
            }
            // ********
            // SEC - Set Carry Flag
            0x38 => {
                self.set_carry();
                self.pc += 1
            }
            // ********
            // SED - Set Decimal Flag
            0xf8 => {
                self.set_decimal();
                self.pc += 1
            }
            // ********
            // SEI - Set Interrupt Disable
            0x78 => {
                self.set_interrupt_disable();
                self.pc += 1
            }
            // ********
            // STA - Store Accumulator
            0x85 => {
                let (_, addr) = self.zero_page();
                self.bus.write_memory(addr, self.accum)
            }
            0x95 => {
                let (_, addr) = self.zero_page_x();
                self.bus.write_memory(addr, self.accum)
            }
            0x8d => {
                let (_, addr) = self.absolute();
                self.bus.write_memory(addr, self.accum)
            }
            0x9d => {
                let (_, addr) = self.absolute_x();
                self.bus.write_memory(addr, self.accum)
            }
            0x99 => {
                let (_, addr) = self.absolute_y();
                self.bus.write_memory(addr, self.accum)
            }
            0x81 => {
                let (_, addr) = self.indirect_x();
                self.bus.write_memory(addr, self.accum)
            }
            0x91 => {
                let (_, addr) = self.indirect_y();
                self.bus.write_memory(addr, self.accum)
            }
            // ********
            // STX - Store X Register
            0x86 => {
                let (_, addr) = self.zero_page();
                self.bus.write_memory(addr, self.rx)
            }
            0x96 => {
                let (_, addr) = self.zero_page_y();
                self.bus.write_memory(addr, self.rx)
            }
            0x8e => {
                let (_, addr) = self.absolute();
                self.bus.write_memory(addr, self.rx)
            }
            // ********
            // STY - Store Y Register
            0x84 => {
                let (_, addr) = self.zero_page();
                self.bus.write_memory(addr, self.ry)
            }
            0x94 => {
                let (_, addr) = self.zero_page_x();
                self.bus.write_memory(addr, self.ry)
            }
            0x8c => {
                let (_, addr) = self.absolute();
                self.bus.write_memory(addr, self.ry)
            }
            // ********
            // TAX - Transfer Accumulator to X
            0xaa => {
                self.rx = self.accum;
                self.cond_set_zero(self.rx == 0);
                self.cond_set_neg(msb(self.rx) == 1);
                self.pc += 1
            }
            // ********
            // TAY - Transfer Accumulator to Y
            0xa8 => {
                self.ry = self.accum;
                self.cond_set_zero(self.ry == 0);
                self.cond_set_neg(msb(self.ry) == 1);
                self.pc += 1
            }
            // ********
            // TSX - Transfer Stack Pointer to X
            0xba => {
                self.rx = self.sp;
                self.cond_set_zero(self.rx == 0);
                self.cond_set_neg(msb(self.rx) == 1);
                self.pc += 1
            }
            // ********
            // TXA - Transfer X to Accumulator
            0x8a => {
                self.accum = self.rx;
                self.cond_set_zero(self.accum == 0);
                self.cond_set_neg(msb(self.accum) == 1);
                self.pc += 1
            }
            // ********
            // TXS - Transfer X to Stack Pointer
            0x9a => {
                self.sp = self.rx;
                self.pc += 1
            }
            // ********
            // TYA - Transfer Y to Accumulator
            0x98 => {
                self.accum = self.ry;
                self.cond_set_zero(self.accum == 0);
                self.cond_set_neg(msb(self.accum) == 1);
                self.pc += 1
            }
            // ********
            _ => {
                panic!("Unexpected opcode found: {:#x}\nSkipping...", opcode)
            }
        }
    }

    fn adc(&mut self, v: u8) {
        let next_accum = self.accum as u16 + (v as u16) + (self.st & CARRY_FLAG) as u16;
        let wrapped_accum = next_accum as u8;

        let overflow = msb(!(self.accum ^ v) & (self.accum ^ wrapped_accum));
        self.set_st_to(OVERFLOW_FLAG - 1, overflow);

        self.cond_set_carry(next_accum > 0xff);

        self.cond_set_zero(wrapped_accum == 0);

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

        let next_v = v.wrapping_shl(1);
        self.cond_set_neg(msb(next_v) == 1);
        self.cond_set_zero(next_v == 0);

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

        // push st with brk flag set
        // there is no need to actually update the cpu st register.
        let p = self.st | 1 << BRK_CMD - 1;
        self.stack_push(p);

        // load IRQ interrupt vector
        let low_addr = self.bus.read_memory(BRK_IH);
        let hi_addr = self.bus.read_memory(BRK_IH + 1);

        let ih_addr = join_hi_low(low_addr, hi_addr);
        self.pc = ih_addr
    }

    fn cmp(&mut self, v: u8) {
        self.cond_set_carry(self.accum >= v);
        self.cond_set_zero(self.accum == v);
        let result = self.accum.wrapping_sub(v);
        self.cond_set_neg(msb(result) == 1);
    }
    fn cpx(&mut self, v: u8) {
        self.cond_set_carry(self.rx >= v);
        self.cond_set_zero(self.rx == v);
        let result = self.rx.wrapping_sub(v);
        self.cond_set_neg(msb(result) == 1);
    }
    fn cpy(&mut self, v: u8) {
        self.cond_set_carry(self.ry >= v);
        self.cond_set_zero(self.ry == v);
        let result = self.ry.wrapping_sub(v);
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
    fn rol(&mut self, v: u8) -> u8 {
        let result = ((v << 1) & 0xfe) | self.get_st(CARRY_FLAG - 1);

        self.set_st_to(CARRY_FLAG - 1, msb(v));
        self.cond_set_neg(msb(result) == 1);
        // TODO do we always set this or only in the case of the A addressing mode?
        self.cond_set_zero(result == 0);
        result
    }
    fn ror(&mut self, v: u8) -> u8 {
        let result = (v >> 1) | self.get_st(CARRY_FLAG - 1) << 7;

        self.set_st_to(CARRY_FLAG - 1, v & 0x01);
        self.cond_set_neg(msb(result) == 1);
        // TODO do we always set this or only in the case of the A addressing mode?
        self.cond_set_zero(result == 0);
        result
    }
    // from https://stackoverflow.com/questions/29193303/6502-emulation-proper-way-to-implement-adc-and-sbc
    fn sbc(&mut self, v: u8) {
        self.adc(!v)
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
    fn zero_page_y(&mut self) -> (u8, u16) {
        let arg = self.bus.read_memory(self.pc + 1);
        let addr = arg.wrapping_add(self.ry);
        let result = self.bus.read_memory(addr as u16);
        self.pc += 2;
        (result, addr as u16)
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
        let addr = join_hi_low(lo, hi).wrapping_add(self.rx as u16);
        let result = self.bus.read_memory(addr);
        self.pc += 3;
        (result, addr)
    }
    fn absolute_y(&mut self) -> (u8, u16) {
        let lo = self.bus.read_memory(self.pc + 1);
        let hi = self.bus.read_memory(self.pc + 2);
        let addr = join_hi_low(lo, hi).wrapping_add(self.ry as u16);
        let result = self.bus.read_memory(addr);
        self.pc += 3;
        (result, addr)
    }
    fn indirect_x(&mut self) -> (u8, u16) {
        let arg = self.bus.read_memory(self.pc + 1);
        let lo = self.bus.read_memory(arg.wrapping_add(self.rx) as u16);
        let hi = self
            .bus
            .read_memory(arg.wrapping_add(self.rx.wrapping_add(1)) as u16);
        let addr = join_hi_low(lo, hi);
        let result = self.bus.read_memory(addr);
        self.pc += 2;
        (result, addr)
    }
    fn indirect_y(&mut self) -> (u8, u16) {
        let arg = self.bus.read_memory(self.pc + 1);
        let lo = self.bus.read_memory(arg as u16);
        let hi = self.bus.read_memory(arg.wrapping_add(1) as u16);
        let addr = join_hi_low(lo, hi).wrapping_add(self.ry as u16);
        let result = self.bus.read_memory(addr);
        self.pc += 2;
        (result, addr)
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
        let bit = v & 0x01;
        if bit == 1 {
            self.st |= v << n
        } else {
            self.st &= !(1 << n)
        }
    }
    fn clear_st(&mut self, n: u8) {
        self.st &= !(1 << n)
    }
    // ********

    /* Stack Functions
    self.sp points to the next value avaiable on the stack.
    If an address (e.g. a u16 value) is "pushed" onto the stack, we should
    remain consistent in that we push the hi byte first, _then_ the low byte.
    */
    fn stack_push(&mut self, byte: u8) {
        let addr = 0x100 + self.sp as u16;
        self.bus.write_memory(addr, byte);
        self.sp = self.sp.wrapping_sub(1);
    }
    fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = 0x100 + self.sp as u16;
        let result = self.bus.read_memory(addr);
        result
    }
    // ********
}

#[cfg(test)]
#[path = "cpu_test.rs"]
mod cpu_test;
