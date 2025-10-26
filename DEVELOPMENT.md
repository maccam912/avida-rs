# Avida-RS Development Log

## Project Overview
Rust implementation of the Avida artificial life simulation with egui GUI. This is a faithful recreation of the original Avida digital evolution platform.

## Architecture

### Core Components

1. **Instruction Set (instruction.rs)**
   - 26 instructions (a-z) matching original Avida
   - Each instruction is a single letter command

2. **CPU (cpu.rs)**
   - 3 registers: AX, BX, CX (32-bit integers)
   - 2 stacks: each max depth 10
   - 4 heads: IP (Instruction Pointer), Read-Head, Write-Head, Flow-Head
   - Input/Output buffers
   - Circular genome memory

3. **Organism (organism.rs)**
   - Genome: Vec<Instruction> (circular)
   - CPU state
   - Merit value (determines CPU cycle allocation)
   - Age, generation tracking
   - Task completion flags

4. **World (world.rs)**
   - 60×60 toroidal grid (wraparound edges)
   - Max 3600 organisms (one per cell)
   - Update scheduler (time-slice based on merit)

5. **Tasks (tasks.rs)**
   - 9 logic operations: NOT, NAND, AND, ORN, OR, ANDN, NOR, XOR, EQU
   - Monitor IO instruction for task detection
   - Merit bonuses: 2^value multiplier

6. **UI (ui.rs)**
   - egui-based visualization
   - Grid view with color-coded cells
   - Statistics panel
   - Organism inspector
   - Controls (speed, pause, mutation rates)

### Default Ancestor
```
wzcagcccccccccccccccccccccccccccccccccccczvfcaxgab
```
- 50 instructions long
- Can self-replicate
- Most are nop-C (placeholder instructions)
- Offspring cost: 189 CPU cycles

## Instruction Set Reference

### No-Operation Instructions
- **nop-A (a)**: No-op, modifies previous instruction or acts as label
- **nop-B (b)**: No-op, modifies previous instruction or acts as label
- **nop-C (c)**: No-op, modifies previous instruction or acts as label

### Flow Control
- **if-n-equ (d)**: Compare BX to complement label; execute next if not equal
- **if-less (e)**: Execute next instruction if BX < complement
- **if-label (y)**: Test if complement label was most recently copied

### Stack Operations
- **pop (f)**: Pop stack top into BX
- **push (g)**: Push BX onto active stack
- **swap-stk (h)**: Toggle between two stacks

### Register Operations
- **swap (i)**: Exchange BX with its complement register
- **shift-r (j)**: Right bit-shift BX (divide by 2)
- **shift-l (k)**: Left bit-shift BX (multiply by 2)
- **inc (l)**: Increment BX
- **dec (m)**: Decrement BX

### Arithmetic
- **add (n)**: BX = BX + CX
- **sub (o)**: BX = BX - CX
- **nand (p)**: BX = BX NAND CX (bitwise)

### I/O
- **IO (q)**: Output BX, check for tasks, input new value

### Genome Management
- **h-alloc (r)**: Allocate memory for offspring
- **h-divide (s)**: Divide organism, create offspring
- **h-copy (t)**: Copy instruction from read-head to write-head (with mutations)
- **h-search (u)**: Find complement label, set BX=distance, CX=size

### Head Movement
- **mov-head (v)**: Jump IP to flow-head position
- **jmp-head (w)**: Move IP by CX register amount
- **get-head (x)**: Copy IP position to CX register
- **set-flow (z)**: Move flow-head to position in CX

## Mutation Rates
- **Copy mutation**: 0.0025 (during h-copy)
- **Insertion**: 0.05 (at division)
- **Deletion**: 0.05 (at division)

## Task Rewards (Default Logic-9 Environment)
```
NOT:  2^1.0 = 2x merit
NAND: 2^1.0 = 2x merit
AND:  2^2.0 = 4x merit
ORN:  2^2.0 = 4x merit
OR:   2^3.0 = 8x merit
ANDN: 2^3.0 = 8x merit
NOR:  2^4.0 = 16x merit
XOR:  2^4.0 = 16x merit
EQU:  2^4.0 = 16x merit
```

## Implementation Status

### ✅ ALL CORE FEATURES COMPLETED!

- [x] Cargo.toml setup with dependencies (egui, eframe, rand)
- [x] Module structure creation
- [x] Instruction enum implementation (all 26 instructions)
- [x] CPU structure and execution engine
- [x] Organism structure with full state
- [x] Self-replication system (h-alloc, h-search, h-copy, h-divide)
- [x] World grid (60×60 toroidal lattice)
- [x] Update scheduler with merit-based CPU allocation
- [x] Task detection (all 9 logic operations)
- [x] Merit system with exponential bonuses
- [x] Mutation engine (copy, insertion, deletion)
- [x] Default ancestor creation
- [x] egui UI with grid visualization
- [x] Statistics panel
- [x] Organism inspector
- [x] Interactive controls
- [x] Successfully builds and runs!

### Build Status
✅ **Project builds successfully** (with minor dead code warnings)
✅ **Application runs** and displays the simulation
✅ **Ancestor organism successfully loads** and begins execution

## File Structure
```
avida-rs/
├── Cargo.toml
├── DEVELOPMENT.md (this file)
├── src/
│   ├── main.rs          - Entry point
│   ├── instruction.rs   - Instruction enum
│   ├── cpu.rs          - Virtual CPU
│   ├── organism.rs     - Organism structure
│   ├── world.rs        - World grid & scheduler
│   ├── tasks.rs        - Task detection & rewards
│   └── ui.rs           - egui interface
```

## Key Implementation Notes

### Template Matching
Instructions use "complements" for template matching:
- nop-A complements nop-B
- nop-B complements nop-C
- nop-C complements nop-A

### h-search Behavior
Searches for complement label sequence. If found:
- BX = distance to label
- CX = size of label
- Flow-head positioned at end of label

### h-copy Mechanism
1. Read instruction at read-head position
2. Apply copy mutation (0.0025 chance of random substitution)
3. Write to write-head position
4. Advance both heads

### h-divide Process
1. Offspring genome = memory region between read-head and write-head
2. Apply insertion mutations (0.05 per position)
3. Apply deletion mutations (0.05 per position)
4. Place offspring in neighbor cell (prefer empty)
5. Offspring inherits parent's merit (before task bonuses)
6. Reset parent's state

### CPU Cycle Allocation
Each update cycle:
1. Calculate total merit in population
2. Allocate CPU cycles proportional to merit
3. Execute instructions until cycles exhausted
4. Organisms with higher merit execute more instructions per update

## Next Steps
1. Create module files (instruction.rs, cpu.rs, etc.)
2. Implement Instruction enum with From<char> trait
3. Implement CPU with circular memory buffer
4. Test instruction execution with unit tests
5. Build organism replication logic
6. Create world grid with toroidal wrapping
7. Add egui visualization

## Research Sources
- github.com/devosoft/avida - Original C++ implementation
- avida-ed.msu.edu - Educational version
- Ofria & Wilke (2004): "Avida: A software platform for research in computational evolutionary biology"

## Notes
- Genome is circular: IP wraps around
- World is toroidal: edges wrap
- Merit determines CPU time allocation (fitness)
- Tasks must be performed via IO instruction
- Ancestor is hand-designed to self-replicate
- Empty nop-C instructions provide mutation buffer
