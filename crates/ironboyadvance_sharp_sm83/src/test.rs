#[cfg(test)]
mod tests {
    use rayon::prelude::*;
    use serde::Deserialize;
    use std::cell::Cell;
    use std::fs;
    use std::path::{Path, PathBuf};

    use crate::{GbMode, cpu::SharpSm83, memory::MemoryInterface};

    /// The suite pads `HALT` and `STOP` out to a fixed three machine cycles instead of
    /// timing them like a normal fetch, so their cycle counts are not compared.
    const HALT_OPCODE: u8 = 0x76;
    const STOP_OPCODE: u8 = 0x10;

    /// Which pins the suite recorded as driven for one M-state.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
    enum BusKind {
        #[serde(rename = "r-m")]
        Read,
        #[serde(rename = "-wm")]
        Write,
        #[serde(rename = "---")]
        Internal,
    }

    /// The address and data pins sampled between two M-states. A `None` pin means the bus
    /// was electrically disconnected from the processor, so its value is a don't-care.
    #[derive(Debug, Deserialize)]
    #[serde(from = "(Option<u16>, Option<u8>, BusKind)")]
    struct BusState {
        address: Option<u16>,
        data: Option<u8>,
        kind: BusKind,
    }

    impl From<(Option<u16>, Option<u8>, BusKind)> for BusState {
        fn from((address, data, kind): (Option<u16>, Option<u8>, BusKind)) -> Self {
            Self { address, data, kind }
        }
    }

    struct TestBus {
        name: String,
        data: Vec<u8>,
        m_cycles: Cell<u64>,
        trace: Vec<BusState>,
        trace_index: Cell<usize>,
    }

    impl TestBus {
        fn new(name: String, trace: Vec<BusState>) -> Self {
            Self {
                name,
                data: vec![0; 0x10000],
                m_cycles: Cell::new(0),
                trace,
                trace_index: Cell::new(0),
            }
        }

        /// Loads take `&self`, so the counter needs interior mutability.
        fn step(&self, m_cycles: u64) {
            self.m_cycles.set(self.m_cycles.get() + m_cycles);
        }

        fn m_cycles(&self) -> u64 {
            self.m_cycles.get()
        }

        fn recorded_m_cycles(&self) -> u64 {
            self.trace.len() as u64
        }

        /// Compares one M-state's pins against the recorded trace, then advances the cursor.
        fn check_bus_state(&self, kind: BusKind, address: Option<u16>, data: Option<u8>) {
            let index = self.trace_index.get();
            self.trace_index.set(index + 1);
            self.step(1);

            let Some(expected) = self.trace.get(index) else {
                panic!("{}: M-state {} ran past the end of the recorded trace", self.name, index);
            };

            assert_eq!(kind, expected.kind, "{}: M-state {} bus kind mismatch", self.name, index);

            if let (Some(address), Some(expected_address)) = (address, expected.address) {
                assert_eq!(
                    address, expected_address,
                    "{}: M-state {} address mismatch",
                    self.name, index
                );
            }

            if let (Some(data), Some(expected_data)) = (data, expected.data) {
                assert_eq!(data, expected_data, "{}: M-state {} data mismatch", self.name, index);
            }
        }

        fn read_untimed(&self, address: u16) -> u8 {
            self.data[address as usize]
        }

        fn write_untimed(&mut self, address: u16, value: u8) {
            self.data[address as usize] = value;
        }
    }

    impl MemoryInterface for TestBus {
        fn load_8(&self, address: u16) -> u8 {
            let value = self.data[address as usize];
            self.check_bus_state(BusKind::Read, Some(address), Some(value));
            value
        }

        fn load_16(&self, address: u16) -> u16 {
            let low = self.load_8(address) as u16;
            let high = self.load_8(address.wrapping_add(1)) as u16;
            high << 8 | low
        }

        fn store_8(&mut self, address: u16, value: u8) {
            self.check_bus_state(BusKind::Write, Some(address), Some(value));
            self.data[address as usize] = value;
        }

        fn store_16(&mut self, address: u16, value: u16) {
            self.store_8(address, value as u8);
            self.store_8(address.wrapping_add(1), (value >> 8) as u8);
        }

        fn idle_cycle(&mut self) {
            // The suite records the last address the CPU drove, which an idle cycle does not
            // hand us, so only the pin kind is checked here.
            self.check_bus_state(BusKind::Internal, None, None);
        }

        fn change_speed(&mut self) {}
    }

    #[derive(Debug, Deserialize)]
    struct State {
        pc: u16,
        sp: u16,
        a: u8,
        b: u8,
        c: u8,
        d: u8,
        e: u8,
        f: u8,
        h: u8,
        l: u8,
        ram: Vec<[u16; 2]>,
    }

    #[derive(Debug, Deserialize)]
    struct Test {
        name: String,
        initial: State,
        #[serde(rename = "final")]
        final_state: State,
        cycles: Vec<BusState>,
    }

    fn run_test_file(path: &Path) -> Result<(), String> {
        let json = fs::read_to_string(path).map_err(|error| format!("Unable to read {:?}: {}", path, error))?;
        let tests: Vec<Test> =
            serde_json::from_str(&json).map_err(|error| format!("Unable to deserialize {:?}: {}", path, error))?;

        for test in tests {
            let name = test.name;
            let initial_state = test.initial;
            let final_state = test.final_state;

            let bus = TestBus::new(name.clone(), test.cycles);
            let mut cpu = SharpSm83::new(bus, false, false, GbMode::Monochrome);

            cpu.registers_mut().set_pc(initial_state.pc);
            cpu.registers_mut().set_sp(initial_state.sp);
            cpu.registers_mut().set_a(initial_state.a);
            cpu.registers_mut().set_b(initial_state.b);
            cpu.registers_mut().set_c(initial_state.c);
            cpu.registers_mut().set_d(initial_state.d);
            cpu.registers_mut().set_e(initial_state.e);
            cpu.registers_mut().set_f(initial_state.f.into());
            cpu.registers_mut().set_h(initial_state.h);
            cpu.registers_mut().set_l(initial_state.l);

            for [address, value] in initial_state.ram {
                cpu.bus_mut().write_untimed(address, value as u8);
            }

            let opcode = cpu.bus().read_untimed(initial_state.pc);

            cpu.cycle();

            assert_eq!(cpu.registers().pc(), final_state.pc, "PC mismatch for test {}", name);
            assert_eq!(cpu.registers().sp(), final_state.sp, "SP mismatch for test {}", name);
            assert_eq!(cpu.registers().a(), final_state.a, "A mismatch for test {}", name);
            assert_eq!(cpu.registers().b(), final_state.b, "B mismatch for test {}", name);
            assert_eq!(cpu.registers().c(), final_state.c, "C mismatch for test {}", name);
            assert_eq!(cpu.registers().d(), final_state.d, "D mismatch for test {}", name);
            assert_eq!(cpu.registers().e(), final_state.e, "E mismatch for test {}", name);
            assert_eq!(
                cpu.registers().f().into_bits(),
                final_state.f,
                "F mismatch for test {}",
                name
            );
            assert_eq!(cpu.registers().h(), final_state.h, "H mismatch for test {}", name);
            assert_eq!(cpu.registers().l(), final_state.l, "L mismatch for test {}", name);

            for [address, value] in final_state.ram {
                assert_eq!(
                    cpu.bus().read_untimed(address),
                    value as u8,
                    "Memory mismatch at {:#06X} for test {}",
                    address,
                    name
                );
            }

            match opcode {
                HALT_OPCODE | STOP_OPCODE => {}
                _ => assert_eq!(
                    cpu.bus().m_cycles(),
                    cpu.bus().recorded_m_cycles(),
                    "Cycle count mismatch for test {}",
                    name
                ),
            }
        }

        Ok(())
    }

    #[test]
    fn single_step_tests() {
        let directory = fs::read_dir("../../external/sm83/v1").expect("Unable to read directory");
        let file_paths: Vec<PathBuf> = directory
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
            .collect();

        // Process files in parallel
        file_paths
            .par_iter()
            .try_for_each(|path| {
                run_test_file(path).map_err(|error| {
                    eprintln!("Test failed in file {:?}: {}", path, error);
                    error
                })
            })
            .expect("Test failures occurred");
    }
}
