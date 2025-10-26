# Avida-RS

A faithful Rust implementation of the Avida artificial life digital evolution platform with an egui-based graphical interface.

## 🧬 What is Avida?

Avida is a software platform for research in digital evolution where self-replicating computer programs (digital organisms) evolve within a user-defined computational environment. Originally developed at Caltech and Michigan State University, Avida has been used in hundreds of scientific publications studying evolution, complexity, and emergence.

## ✨ Features

- **Complete Avida Instruction Set**: All 26 instructions (a-z) from the original Avida
- **Virtual CPU**: 3 registers, 2 stacks, 4 heads (IP, Read, Write, Flow)
- **Self-Replication**: Organisms copy their genome and produce offspring with mutations
- **60×60 Toroidal World**: Wraparound grid topology
- **Task System**: 9 logic operations (NOT, NAND, AND, ORN, OR, ANDN, NOR, XOR, EQU)
- **Merit-Based Selection**: CPU cycles allocated proportionally to task performance
- **Mutation System**: Copy mutations (0.0025), insertions (0.05), deletions (0.05)
- **Real-Time Visualization**: Watch evolution unfold in real-time
- **Interactive Inspector**: Click organisms to view their genome, CPU state, and tasks
- **Configurable Parameters**: Adjust mutation rates and simulation speed

## 🚀 Quick Start

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

### Building

```bash
cargo build --release
```

### Running

```bash
cargo run --release
```

## 🎮 Using the Simulator

### Controls

- **▶ Play / ⏸ Pause**: Start/stop the simulation
- **Speed Slider**: Control updates per frame (1-100)
- **Reset**: Clear world and inject new ancestor
- **Click on cells**: Inspect individual organisms

### Display Modes

- **Age**: Color by organism age (purple → red)
- **Merit**: Color by task performance (yellow)
- **Genome Size**: Red = larger, Blue = smaller than ancestor
- **Tasks Completed**: Green intensity = number of tasks

### Statistics Panel

- Population size and births/deaths
- Average genome size and merit
- Mutation rate controls
- Task completion counts

### Inspector Panel

- Organism position and generation
- Complete genome sequence
- CPU state (registers, heads)
- Tasks completed

## 🧬 The Default Ancestor

The simulation starts with the hand-designed ancestor organism:

```
wzcagcccccccccccccccccccccccccccccccccccczvfcaxgab
```

This 50-instruction genome is capable of self-replication. Most instructions are `c` (nop-C), serving as a mutation buffer and allowing evolution to explore new functionality.

## 📊 How Evolution Works

1. **Replication**: Organisms execute instructions from their circular genome
2. **Self-Copy**: Using h-alloc, h-search, h-copy, and h-divide instructions
3. **Mutations**: Random changes during copying and division
4. **Tasks**: Organisms perform logic operations to gain merit
5. **Selection**: Higher merit = more CPU cycles = faster reproduction
6. **Evolution**: Beneficial mutations spread through the population

## 🔬 Task System

Organisms gain merit bonuses by performing logic operations:

| Task | Operation | Merit Multiplier |
|------|-----------|------------------|
| NOT  | !B        | 2× (2^1.0)      |
| NAND | !(A & B)  | 2× (2^1.0)      |
| AND  | A & B     | 4× (2^2.0)      |
| ORN  | A \| !B   | 4× (2^2.0)      |
| OR   | A \| B    | 8× (2^3.0)      |
| ANDN | A & !B    | 8× (2^3.0)      |
| NOR  | !(A \| B) | 16× (2^4.0)     |
| XOR  | A ^ B     | 16× (2^4.0)     |
| EQU  | !(A ^ B)  | 16× (2^4.0)     |

Tasks are detected through the `IO` instruction (q), which outputs values and receives inputs.

## 🏗️ Architecture

### Core Modules

- **instruction.rs**: 26-instruction enum with parsing
- **cpu.rs**: Virtual CPU with registers, stacks, and heads
- **organism.rs**: Organism structure with genome and state
- **execute.rs**: Instruction execution engine
- **tasks.rs**: Task detection and merit rewards
- **world.rs**: 60×60 toroidal grid and update scheduler
- **ui.rs**: egui-based visualization and controls

### Key Concepts

- **Circular Genome**: Instructions wrap around (IP % genome_size)
- **Template Matching**: Nop complements used for h-search
  - nop-A ↔ nop-B
  - nop-B ↔ nop-C
  - nop-C ↔ nop-A
- **Merit System**: CPU cycles ∝ merit / total_merit
- **Toroidal Topology**: Edges wrap (organisms born at neighbors)

## 📚 Research Background

Based on:
- Ofria, C., & Wilke, C. (2004). "Avida: A software platform for research in computational evolutionary biology." *Artificial Life*, 10, 191-229.
- Original implementation: [github.com/devosoft/avida](https://github.com/devosoft/avida)
- Educational version: [avida-ed.msu.edu](https://avida-ed.msu.edu)

## 🛠️ Technical Details

### Instruction Set Summary

| Range | Type | Instructions |
|-------|------|--------------|
| a-c | Nops | No-ops, labels, register selection |
| d-e | Conditionals | if-n-equ, if-less |
| f-h | Stack | pop, push, swap-stk |
| i-m | Register Ops | swap, shift-r, shift-l, inc, dec |
| n-p | Arithmetic | add, sub, nand |
| q | I/O | IO (task detection) |
| r-u | Genome | h-alloc, h-divide, h-copy, h-search |
| v-x | Head Movement | mov-head, jmp-head, get-head |
| y-z | Flow Control | if-label, set-flow |

### Mutation Rates (Default)

- **Copy Mutation**: 0.0025 (during h-copy)
- **Insertion**: 0.05 (per position at division)
- **Deletion**: 0.05 (per position at division)

### World Parameters

- **Grid Size**: 60×60 (3,600 max organisms)
- **Topology**: Toroidal (wraparound edges)
- **Updates**: Merit-based CPU cycle allocation
- **Birth Method**: Prefer empty neighbors, else random

## 🔍 What to Watch For

As the simulation runs, you may observe:

1. **Population Growth**: Ancestor replicates and fills the world
2. **Genome Size Changes**: Mutations cause length variation
3. **Task Evolution**: Organisms evolve logic operations
4. **Merit Increase**: Average merit rises as tasks are discovered
5. **Diversity**: Different lineages with varying genomes
6. **Parasites**: Organisms that copy from neighbors (sometimes)

## 🐛 Known Issues / Future Enhancements

Current implementation is feature-complete for basic digital evolution. Potential additions:

- [ ] Save/load organism genomes
- [ ] Phylogenetic tree visualization
- [ ] Detailed lineage tracking
- [ ] More task types
- [ ] Energy model
- [ ] Population structure analysis
- [ ] Export statistics to CSV

## 📄 License

This is an educational reimplementation. The original Avida is developed by the Devolab at Michigan State University.

## 🙏 Acknowledgments

- Charles Ofria and the Avida development team
- Original Avida: [devosoft/avida](https://github.com/devosoft/avida)
- egui framework: [emilk/egui](https://github.com/emilk/egui)

## 📞 Support

For issues or questions, refer to:
- DEVELOPMENT.md for implementation details
- Original Avida documentation: [avida.devosoft.org](http://avida.devosoft.org)
- Avida-ED tutorials: [avida-ed.msu.edu](https://avida-ed.msu.edu)

---

**Happy Evolving! 🧬**
