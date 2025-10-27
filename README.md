# Avida-RS

Avida-RS is a Rust reimplementation of the Avida digital evolution platform featuring an egui-based visualizer. It simulates self-replicating digital organisms that compete for resources while mutating and evolving over time.

## Features
- Complete Avida instruction set with circular genomes and template matching
- Merit-driven scheduling that rewards organisms for performing logic tasks
- Toroidal 60×60 world grid with live visualization and organism inspector
- Adjustable mutation rates and simulation speed for experimentation

## Getting Started
1. Install the latest stable [Rust toolchain](https://www.rust-lang.org/tools/install).
2. Build the project:
   ```bash
   cargo build --release
   ```
3. Run the simulator:
   ```bash
   cargo run --release
   ```

Launching the binary opens the egui interface where you can pause/play the simulation, change the update rate, and inspect individual organisms.

## Project Structure
- `src/main.rs` – Application entry point that launches the egui visualizer.
- `src/lib.rs` – Module exports shared by the main and debug binaries.
- `src/bin/debug_test.rs` – Diagnostics-oriented binary for targeted experiments.
- `src/instruction.rs` – Instruction enum and parsing helpers.
- `src/cpu.rs` – Virtual CPU with registers, stacks, and execution heads.
- `src/execute.rs` – Instruction execution pipeline connecting organisms with tasks.
- `src/organism.rs` – Organism state, genome storage, and replication logic.
- `src/world.rs` – Population grid, scheduler, and mutation handling.
- `src/tasks.rs` – Logic task detection and merit rewards.
- `src/ui.rs` – egui front-end and control panels.
- `src/debug.rs` – Logging and tracing utilities used by the simulator.
- `src/diagnostics.rs` – Population analysis helpers and reporting utilities.

## Documentation
This README provides the primary quickstart. Module-level Rustdoc comments and examples in `src/bin/debug_test.rs` and `src/diagnostics.rs` document the latest inspection and analysis tools.

Contributions, issue reports, and experiment write-ups are welcome!
