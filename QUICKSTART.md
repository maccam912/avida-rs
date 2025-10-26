# Avida-RS Quick Start Guide

## 🎯 Goal

Watch digital organisms evolve in real-time! This simulation demonstrates natural selection, mutation, and the evolution of complexity in a simple artificial life system.

## 📋 Prerequisites

Install Rust from [rustup.rs](https://rustup.rs/)

## ⚡ Run in 3 Steps

1. **Build the project**
   ```bash
   cd avida-rs
   cargo build --release
   ```

2. **Run the simulator**
   ```bash
   cargo run --release
   ```

3. **Watch evolution happen!**
   - The simulation starts with a single ancestor organism (center of grid)
   - Green cells = organisms that have evolved to perform tasks
   - Watch the population grow and evolve

## 🎮 Basic Controls

| Control | Action |
|---------|--------|
| **Play/Pause button** | Start/stop simulation |
| **Speed slider** | Make evolution faster (1-100 updates/frame) |
| **Click on a cell** | Inspect organism's genome and state |
| **Reset button** | Start over with fresh ancestor |

## 👀 What to Look For

### First 10 seconds
- Ancestor begins self-replication
- Population starts growing from center
- Grid fills with organisms (dark cells)

### After 30 seconds (at speed ~10)
- Some organisms turn **green** (evolved tasks!)
- Population reaches capacity (3,600 organisms)
- Average merit increases (shown in stats panel)

### After 1 minute
- **Merit diversity**: Different shades of green
- **Genome variation**: Click organisms to see different genomes
- **Task evolution**: Check which logic operations evolved

## 🔬 Understanding the Display

### Color Modes (change in left panel)

**Tasks Mode (default - green)**
- Darker = no tasks
- Brighter green = more tasks completed
- Brightest = all 9 tasks (rare!)

**Merit Mode (yellow)**
- Brighter = better at getting CPU time
- Result of completing tasks

**Age Mode (purple-red)**
- Shows how old each organism is
- Cycling colors

**Genome Size (red/blue)**
- Red = larger than ancestor (50 instructions)
- Blue = smaller than ancestor
- Most evolution keeps genome near 50

## 📊 Reading the Statistics

**Left Panel Shows:**
- **Population**: Current number of organisms
- **Total Births**: How many organisms created
- **Avg Genome Size**: Typical genome length (starts at 50)
- **Avg Merit**: Population fitness (starts at 1.0, increases with tasks)
- **Task counts**: How many organisms can do each operation

**Tasks List:**
- NOT (2× merit) - Simplest, usually evolves first
- NAND (2×) - Also simple
- AND through EQU (4×-16×) - Harder, evolve later

## 🧬 The Ancestor Genome

```
wzcagcccccccccccccccccccccccccccccccccccczvfcaxgab
```

- 50 instructions (letters a-z)
- Can copy itself perfectly (without mutations)
- Mostly `c` (no-op) = "junk DNA" for mutations to work with
- Essential replication instructions: `w`, `z`, `a`, `g`, `r`, `s`, `t`, `u`

## 🎓 Try These Experiments

### 1. Evolution Speed Test
- Set speed to **1** (slow motion)
- Watch a single organism replicate step-by-step
- See the copy process in action

### 2. Mutation Effects
- Adjust **Copy mutation** slider (try 0.01 = higher)
- More mutations = more diversity but also more death
- Try **Insertion/Deletion** rates (default 0.05)

### 3. Task Evolution
- Start simulation at speed 10-20
- Watch "Tasks Completed" panel
- Usually NOT or NAND evolves first (2× bonus)
- Complex tasks (16×) take longer

### 4. Genome Inspector
- Let simulation run for 30+ seconds
- Click a bright green organism
- Look at its genome - compare to ancestor
- Find the evolved instructions that compute logic

## ❓ FAQ

**Q: Nothing is happening?**
- Make sure you clicked Play (▶ button)
- Increase the speed slider
- Check that population > 0 in stats

**Q: Why did the population crash?**
- High mutation rates can be lethal
- Reset mutation rates to defaults
- Click Reset to restart

**Q: Why are some organisms white/bright?**
- They've completed many tasks
- Higher merit = more CPU time
- They reproduce faster = evolutionary success!

**Q: How long until I see tasks?**
- Usually 20-60 seconds at medium speed
- NOT and NAND typically appear first
- Complex tasks (XOR, EQU) may take minutes

**Q: Can I save interesting organisms?**
- Not yet! This is a future feature
- For now, you can copy genome from inspector

## 🎯 Success Criteria

You've successfully run Avida when you see:

✅ Population grows to 1000+
✅ At least one task shows count > 0
✅ Average merit increases above 1.0
✅ Green organisms appear in the grid
✅ Inspector shows varied genomes when clicking cells

## 🚀 Next Steps

Once you're comfortable with the basics:

1. Read **README.md** for full feature documentation
2. Check **DEVELOPMENT.md** for implementation details
3. Experiment with extreme mutation rates
4. Try to evolve all 9 tasks
5. Observe population dynamics over long runs

## 📚 Learn More

- [Original Avida](http://avida.devosoft.org) - Scientific research platform
- [Avida-ED](https://avida-ed.msu.edu) - Educational version
- Key Paper: Ofria & Wilke (2004) "Avida: A software platform for research in computational evolutionary biology"

---

**Have fun watching evolution in action! 🧬✨**
