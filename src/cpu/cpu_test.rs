use crate::cartridge::Cartridge;

use super::CPU;

fn make_cpu_with_rom(seed_rom: &[u8], initial_pc: u16) -> CPU {
    let mut cpu = CPU::new();
    cpu.pc = initial_pc;

    let mut buff = [0; 0x4000];

    seed_rom.iter().enumerate().for_each(|(idx, byte)| {
        buff[idx] = *byte;
    });

    cpu.load_cartridge(Cartridge { bytes: buff });

    cpu
}

fn execute_n_instructions(cpu: &mut CPU, n: u16) {
    (0..n).for_each(|_| {
        let opcode = cpu.bus.read_memory(cpu.pc);
        cpu.exec_opcode(opcode)
    })
}

#[test]
fn test_set_st_to() {
    let mut cpu = CPU::new();
    let b1 = 0b111;
    let b2 = 0b101;
    cpu.st = b1;
    cpu.set_st_to(1, 0);
    assert_eq!(cpu.st, 0b101);
    cpu.st = b2;
    cpu.set_st_to(1, 1);
    assert_eq!(cpu.st, 0b111);
}

#[test]
fn test_bit() {
    let mut cpu = CPU::new();
    cpu.st = 0;

    cpu.bit(0b11000000);

    let actual = cpu.st;
    println!("{actual:b}");

    assert_eq!(0b11000010, cpu.st)
}

#[test]
fn test_lda_sta_bit_test() {
    let rom = vec![
        0xa9, /* LDA #$00 */
        0x00, 0x85, /* STA $01 */
        0x01, 0x24, /* BIT $01 */
        0x01,
    ];
    let mut cpu = make_cpu_with_rom(&rom, 0xc000);
    cpu.accum = 0xff;
    cpu.st = 0xe4;
    cpu.sp = 0xfb;

    execute_n_instructions(&mut cpu, 3);
    println!("Expected st {:#x}, actual st: {:#x}", 0x26, cpu.st);
    assert_eq!(cpu.st, 0x26);
    assert_eq!(cpu.pc, 0xc006)
}

#[test]
fn test_adc() {
    let rom = vec![0x69 /* ADC #$69 */, 0x69];

    let mut cpu = make_cpu_with_rom(&rom, 0xc000);
    cpu.st = 0x6e;

    execute_n_instructions(&mut cpu, 1);

    println!("Expected st {:#x}, actual st: {:#x}", 0x2c, cpu.st);
    assert_eq!(cpu.st, 0x2c)
}
