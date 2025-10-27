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
- `src/instruction.rs` – Instruction enum and parsing helpers
- `src/cpu.rs` – Virtual CPU with registers, stacks, and execution heads
- `src/organism.rs` – Organism state, genome storage, and replication logic
- `src/world.rs` – Population grid, scheduler, and mutation handling
- `src/tasks.rs` – Logic task detection and merit rewards
- `src/ui.rs` – egui front-end and control panels

## Documentation
Additional reference material lives alongside the repository:
- `QUICKSTART.md` – Step-by-step usage and interface overview
- `OPTIMIZATIONS.md` – Ideas for future performance and feature work
- `SESSION_SUMMARY.md` – Recent development context and follow-up items
- `FIX_SUMMARY.md` – Latest resolved issues and verification notes

Contributions, issue reports, and experiment write-ups are welcome!
