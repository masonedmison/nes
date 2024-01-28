use regex::Regex;
use std::{default, fs};

use crate::{
    bus::Bus,
    cartridge::{Cartridge, Mirroring},
    debug::CpuState,
    ppu::PPU,
};

use super::CPU;

fn make_cpu_with_empty_bus() -> CPU {
    let bus = Bus::new(PPU::new());
    CPU::new(bus)
}

// run and compare nestest.nes against nestest.log
fn run_debug_until(cpu: &mut CPU, n: u32) -> Vec<CpuState> {
    cpu.pc = 0xc000;
    cpu.st = 0x24;
    cpu.cycles = 7;

    let mut states: Vec<CpuState> = vec![];
    let mut start_cycles;
    let mut state: CpuState;
    let mut i = 0;
    while i < n {
        start_cycles = cpu.cycles;
        cpu.stack_pop_count = 0;
        cpu.stack_push_count = 0;

        let opcode = cpu.bus.read_memory(cpu.pc);
        cpu.cycles += 1;

        state = cpu.debug_exec(opcode);
        states.push(state);

        // Make sure to check cycle diff count _before_ applying
        // any cycles due to accessing the stack
        if cpu.cycles - start_cycles == 1 {
            cpu.cycles += 1
        }
        // TODO don't love this...
        cpu.cycles += (cpu.stack_pop_count + cpu.stack_push_count) as u64;

        i += 1
    }
    states
}
fn parse_nestest_log() -> Vec<CpuState> {
    /*
    Groups:
    1: Address, 2: opcode (as hex), 3: accum, 4: rx, 5: ry, 6: st, 7: sp, 8: cycle count
    */
    let re =
        Regex::new(r"(\w{4})\s+(\w{2}).*A:(\w+)\sX:(\w+)\sY:(\w+)\sP:(\w+)\sSP:(\w+).*CYC:(\w+)")
            .unwrap();
    let neslog = fs::read_to_string("./test_roms/logs/nestest.log").unwrap();
    let mut states: Vec<CpuState> = vec![];
    for (idx, line) in neslog.split("\n").enumerate() {
        if line.is_empty() {
            continue;
        }

        let caps = re.captures(line).expect(&format!(
            "Expected match but failed at line: {}\nEntry: {}",
            idx, line
        ));
        let state = CpuState {
            addr: u16::from_str_radix(&caps[1], 16).unwrap(),
            opcode: u8::from_str_radix(&caps[2], 16).unwrap(),
            a: u8::from_str_radix(&caps[3], 16).unwrap(),
            x: u8::from_str_radix(&caps[4], 16).unwrap(),
            y: u8::from_str_radix(&caps[5], 16).unwrap(),
            p: u8::from_str_radix(&caps[6], 16).unwrap(),
            sp: u8::from_str_radix(&caps[7], 16).unwrap(),
            cycles: *&caps[8].parse().unwrap(),
        };
        states.push(state)
    }
    states
}
#[test]
fn nestest() {
    let file_path = "./test_roms/cpu/nestest.nes";
    let cartridge = Cartridge::load(file_path).expect("Error loading file");
    let mut cpu = make_cpu_with_empty_bus();
    cpu.load_cartridge(cartridge);

    let actual = run_debug_until(&mut cpu, 5003);
    let expected = parse_nestest_log();

    let mut prev: (CpuState, CpuState) = (CpuState::default(), CpuState::default());
    for (idx, (a, e)) in actual.into_iter().zip(expected).enumerate() {
        assert_eq!(
            a,
            e,
            "Discrepancy found at index {}\nA: {}\nE: {}",
            idx,
            prev.0.render(),
            prev.1.render()
        );
        prev = (a, e);
    }
}
