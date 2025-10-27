use crate::cpu::CPU;
use crate::instruction::Instruction;

/// Represents a digital organism in Avida
#[derive(Debug, Clone)]
pub struct Organism {
    /// The organism's genome (circular sequence of instructions)
    pub genome: Vec<Instruction>,

    /// Virtual CPU state
    pub cpu: CPU,

    /// Merit value - determines CPU cycle allocation
    /// Higher merit = more CPU cycles per update
    /// Merit is ONLY increased by completing tasks
    pub merit: f64,

    /// Total number of instructions executed by this organism
    pub instruction_count: u64,

    /// Generation number (0 for injected ancestors)
    pub generation: u32,

    /// Number of offspring produced by this organism
    pub offspring_count: u32,

    /// Tasks completed (bit flags for 9 tasks)
    /// Each bit represents whether task 0-8 has been completed
    pub tasks_completed: u16,

    /// Child genome being constructed (allocated memory)
    pub child_genome: Option<Vec<Instruction>>,

    /// Highest offset copied into the child genome during current gestation
    pub child_copy_progress: usize,

    /// Number of instructions executed during current gestation period
    /// (reset after each successful division)
    pub gestation_cycles: u64,

    /// Position in the world grid
    pub position: Option<(usize, usize)>,
}

impl Organism {
    /// Create a new organism with the given genome
    /// All organisms start with merit 1.0 (base merit)
    pub fn new(genome: Vec<Instruction>) -> Self {
        Self {
            genome,
            cpu: CPU::new(),
            merit: 1.0, // Base merit - ONLY increased by completing tasks
            instruction_count: 0,
            generation: 0,
            offspring_count: 0,
            tasks_completed: 0,
            child_genome: None,
            child_copy_progress: 0,
            gestation_cycles: 0,
            position: None,
        }
    }

    /// Create the default ancestor organism
    /// This genome successfully self-replicates
    pub fn ancestor() -> Self {
        // Working ancestor genome structure:
        // r     - h-alloc: allocate memory
        // u     - h-search (no template): sets flow-head to next instruction
        // t     - h-copy: copy one instruction (in loop)
        // yab   - if-label ab: check if complement 'bc' was just copied
        // s     - h-divide: divide (only if end template found)
        // va    - mov-head with nop-A: jump IP back to loop start
        // c...  - padding nops (buffer for mutations)
        // bc    - end template marker

        let genome_str = "rutyabsvacccccccccccccccccccccccccccccccccccccccbc";

        let genome =
            crate::instruction::parse_genome(genome_str).expect("Ancestor genome should be valid");
        Self::new(genome)
    }

    /// Create the Avida-ED style ancestor (for reference - doesn't self-replicate!)
    /// This is the educational version genome that we found in research
    #[allow(dead_code)]
    pub fn ancestor_avida_ed() -> Self {
        let genome_str = "wzcagcccccccccccccccccccccccccccccccccccczvfcaxgab";
        let genome =
            crate::instruction::parse_genome(genome_str).expect("Ancestor genome should be valid");
        Self::new(genome)
    }

    /// Create a task-capable ancestor with "junk DNA" for evolution
    pub fn ancestor_with_tasks() -> Self {
        let genome_str = "rutyabsvagqfgqpgqnocccccccccccccccccccccccccccccbc";

        let genome = crate::instruction::parse_genome(genome_str)
            .expect("Task-capable ancestor genome should be valid");
        Self::new(genome)
    }

    /// Get the current instruction at IP
    pub fn current_instruction(&self) -> Option<Instruction> {
        self.genome.get(self.cpu.ip).copied()
    }

    /// Advance the instruction pointer with circular wrapping
    pub fn advance_ip(&mut self) {
        self.cpu.ip = (self.cpu.ip + 1) % self.genome.len();
    }

    /// Execute a single instruction and increment counters
    /// This should be called by the world scheduler
    pub fn execute_instruction(&mut self) {
        self.instruction_count += 1;
        self.gestation_cycles += 1;
    }

    /// Allocate memory for offspring (h-alloc instruction)
    pub fn allocate_child(&mut self) {
        if self.child_genome.is_some() {
            crate::debug::log_event(format!(
                "[WARN] h-alloc called but child already exists (gen:{}, cycles:{}, ip:{})",
                self.generation, self.instruction_count, self.cpu.ip
            ));
            return;
        }

        let child_size = self.genome.len();
        // Initialize with nop-A instructions
        self.child_genome = Some(vec![Instruction::NopA; child_size]);

        // Set AX register to original size
        self.cpu.registers[0] = child_size as i32;

        // Reset write head to start of child genome
        self.cpu.write_head = 0;
        self.cpu.read_head = 0;
        self.cpu.last_copied_label.clear();
        self.child_copy_progress = 0;

        crate::debug::ALLOCATIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        crate::debug::log_event(format!(
            "[ALLOC] gen:{} cycles:{} size:{} ip:{} -> child allocated",
            self.generation, self.instruction_count, child_size, self.cpu.ip
        ));
    }

    /// Copy an instruction from read-head to write-head (h-copy instruction)
    /// Returns the instruction that was copied (after potential mutation)
    pub fn copy_instruction(&mut self, mutation_rate: f64) -> Option<Instruction> {
        use rand::Rng;

        if let Some(child) = &mut self.child_genome {
            // Check bounds before reading
            if self.cpu.read_head >= self.genome.len() {
                crate::debug::log_event(format!(
                    "[ERROR] h-copy read_head out of bounds! rh:{} genome_len:{}",
                    self.cpu.read_head,
                    self.genome.len()
                ));
                return None;
            }

            // Read instruction at read-head
            let inst = self.genome.get(self.cpu.read_head).copied()?;

            // Apply copy mutation
            let mut mutated = false;
            let inst_to_write = if rand::thread_rng().gen::<f64>() < mutation_rate {
                // Random mutation
                let random_char = (b'a' + rand::thread_rng().gen_range(0..26)) as char;
                mutated = true;
                Instruction::from_char(random_char).unwrap_or(inst)
            } else {
                inst
            };

            // Write to child genome if within bounds
            if self.cpu.write_head < child.len() {
                child[self.cpu.write_head] = inst_to_write;

                let copies =
                    crate::debug::COPIES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                // Log first few copies and every 50th copy
                if copies < 10 || copies % 50 == 0 {
                    crate::debug::log_event(format!(
                        "[COPY #{}] rh:{} -> wh:{} inst:{} progress:{}/{} {}",
                        copies,
                        self.cpu.read_head,
                        self.cpu.write_head,
                        inst_to_write.to_char(),
                        self.cpu.write_head + 1,
                        child.len(),
                        if mutated { "MUTATED" } else { "" }
                    ));
                }
            } else {
                crate::debug::log_event(format!(
                    "[WARN] h-copy write_head out of bounds! wh:{} child_len:{}",
                    self.cpu.write_head,
                    child.len()
                ));
            }

            // Update last copied label if it's a nop
            if inst_to_write.is_nop() {
                self.cpu.last_copied_label.push(inst_to_write);
            } else {
                self.cpu.last_copied_label.clear();
            }

            // Advance both heads
            self.cpu.read_head = self.cpu.advance_head(self.cpu.read_head, self.genome.len());
            self.cpu.write_head += 1;
            self.child_copy_progress = self.child_copy_progress.max(self.cpu.write_head);

            Some(inst_to_write)
        } else {
            crate::debug::log_event(format!(
                "[ERROR] h-copy called but no child allocated! gen:{} cycles:{} ip:{}",
                self.generation, self.instruction_count, self.cpu.ip
            ));
            None
        }
    }

    /// Divide the organism and return the offspring
    /// The parent resets its gestation cycle counter
    /// The offspring starts with merit 1.0 (task bonuses not inherited)
    pub fn divide(&mut self, insertion_rate: f64, deletion_rate: f64) -> Option<Organism> {
        use rand::Rng;

        let child_genome_opt = self.child_genome.take();
        if child_genome_opt.is_none() {
            crate::debug::FAILED_DIVISIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            crate::debug::log_event(format!(
                "[FAIL] h-divide called but no child allocated! gen:{} cycles:{}",
                self.generation, self.instruction_count
            ));
            return None;
        }

        let mut child_genome = child_genome_opt.unwrap();

        // Check if enough was copied (at least 50% of parent genome)
        let progress = self.child_copy_progress.min(child_genome.len());
        let parent_size = self.genome.len();

        if progress < parent_size / 2 {
            crate::debug::FAILED_DIVISIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            crate::debug::log_event(format!(
                "[FAIL] h-divide with insufficient copying! progress:{}/{} (<50%)",
                progress, parent_size
            ));
            // Put child back for retry
            self.child_genome = Some(child_genome);
            return None;
        }

        // Trim to write-head position
        child_genome.truncate(progress);
        let size_before_mutations = child_genome.len();

        // Apply insertion and deletion mutations
        // insertion_rate/deletion_rate is the probability of ONE mutation per division
        let mut insertions = 0;
        let mut deletions = 0;
        let mut rng = rand::thread_rng();

        // Single deletion check per division
        if !child_genome.is_empty() && rng.gen::<f64>() < deletion_rate {
            // Choose random position to delete
            let pos = rng.gen_range(0..child_genome.len());
            child_genome.remove(pos);
            deletions = 1;
        }

        // Single insertion check per division
        if rng.gen::<f64>() < insertion_rate {
            // Choose random position to insert
            let pos = if child_genome.is_empty() {
                0
            } else {
                rng.gen_range(0..=child_genome.len())
            };
            let random_char = (b'a' + rng.gen_range(0..26)) as char;
            if let Some(inst) = Instruction::from_char(random_char) {
                child_genome.insert(pos, inst);
                insertions = 1;
            }
        }

        // Ensure minimum genome size
        if child_genome.is_empty() {
            child_genome.push(Instruction::NopC);
            crate::debug::log_event("[WARN] Child genome was empty after mutations, added NopC");
        }

        let final_size = child_genome.len();

        // Create offspring
        let mut offspring = Organism::new(child_genome);
        offspring.generation = self.generation + 1;
        offspring.merit = 1.0; // Start with base merit (task bonuses not inherited)

        let divisions = crate::debug::DIVISIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // Log first few divisions and every 10th
        if divisions < 5 || divisions % 10 == 0 {
            crate::debug::log_event(format!(
                "[DIVIDE #{}] gen:{}->{} parent_size:{} copied:{} final:{} (ins:{} del:{}) parent_merit:{:.1} child_merit:{:.1} gestation:{}",
                divisions,
                self.generation,
                offspring.generation,
                parent_size,
                size_before_mutations,
                final_size,
                insertions,
                deletions,
                self.merit,
                offspring.merit,
                self.gestation_cycles
            ));
        }

        // Update parent
        self.offspring_count += 1;

        // Reset parent state for next replication cycle
        self.gestation_cycles = 0;
        self.cpu.ip = 0; // Reset to start of genome
        self.cpu.read_head = 0;
        self.cpu.write_head = 0;
        self.cpu.last_copied_label.clear();
        self.child_copy_progress = 0;

        Some(offspring)
    }

    /// Check if a task bit is set
    pub fn has_completed_task(&self, task_index: u8) -> bool {
        if task_index >= 9 {
            return false;
        }
        (self.tasks_completed & (1 << task_index)) != 0
    }

    /// Set a task as completed
    pub fn complete_task(&mut self, task_index: u8) {
        if task_index < 9 {
            self.tasks_completed |= 1 << task_index;
        }
    }

    /// Get genome as a string
    pub fn genome_string(&self) -> String {
        crate::instruction::genome_to_string(&self.genome)
    }

    /// Get the size of the genome
    pub fn genome_size(&self) -> usize {
        self.genome.len()
    }

    /// Calculate fitness (merit divided by gestation time)
    /// This is for statistics only - scheduling uses merit directly
    pub fn fitness(&self) -> f64 {
        if self.gestation_cycles == 0 {
            return self.merit;
        }
        self.merit / (self.gestation_cycles as f64)
    }

    /// Get age (total instructions executed)
    pub fn age(&self) -> u64 {
        self.instruction_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ancestor_creation() {
        let ancestor = Organism::ancestor();
        assert_eq!(ancestor.genome.len(), 50);
        assert_eq!(ancestor.merit, 1.0);
        assert_eq!(ancestor.generation, 0);
        assert_eq!(ancestor.instruction_count, 0);
    }

    #[test]
    fn test_organism_from_genome() {
        let genome = vec![Instruction::NopA, Instruction::NopB, Instruction::Add];
        let org = Organism::new(genome);
        assert_eq!(org.genome.len(), 3);
        assert_eq!(org.cpu.ip, 0);
        assert_eq!(org.merit, 1.0);
    }

    #[test]
    fn test_allocate_child() {
        let mut org = Organism::ancestor();
        org.allocate_child();
        assert!(org.child_genome.is_some());
        assert_eq!(org.child_genome.as_ref().unwrap().len(), 50);
        assert_eq!(org.cpu.registers[0], 50);
        assert_eq!(org.child_copy_progress, 0);
    }

    #[test]
    fn test_task_completion() {
        let mut org = Organism::ancestor();
        assert!(!org.has_completed_task(0));
        org.complete_task(0);
        assert!(org.has_completed_task(0));
        assert!(!org.has_completed_task(1));
    }

    #[test]
    fn test_copy_instruction() {
        let mut org = Organism::ancestor();
        org.allocate_child();
        org.cpu.read_head = 0;
        org.cpu.write_head = 0;

        let copied = org.copy_instruction(0.0); // No mutations
        assert!(copied.is_some());
        assert_eq!(org.cpu.read_head, 1);
        assert_eq!(org.cpu.write_head, 1);
        assert_eq!(org.child_copy_progress, 1);
    }

    #[test]
    fn test_divide_retains_copied_genome() {
        let mut org = Organism::ancestor();
        let genome_len = org.genome_size();

        org.allocate_child();

        for _ in 0..genome_len {
            let copied = org.copy_instruction(0.0);
            assert!(
                copied.is_some(),
                "copy_instruction returned None before genome copied"
            );
        }

        assert!(org.child_genome.is_some());
        assert_eq!(org.cpu.write_head, genome_len);

        let offspring = org.divide(0.0, 0.0).expect("offspring should be produced");
        assert_eq!(offspring.genome_size(), genome_len);
        assert!(org.child_genome.is_none());
        assert_eq!(org.cpu.write_head, 0);
        assert_eq!(org.child_copy_progress, 0);
    }

    #[test]
    fn test_instruction_driven_reproduction_preserves_genome() {
        use crate::execute::execute_instruction;
        use crate::tasks::TaskDetector;

        let mut org = Organism::ancestor();
        let original_genome = org.genome.clone();
        let genome_len = original_genome.len();
        let mut detector = TaskDetector::new();

        let mut steps = 0;
        let offspring = loop {
            let (should_divide, _) = execute_instruction(&mut org, &mut detector, 0.0);
            steps += 1;

            if should_divide {
                break org.divide(0.0, 0.0);
            }

            assert!(
                steps <= 2000,
                "Ancestor failed to divide within 2000 instructions"
            );
        };

        let offspring = offspring.expect("division should yield offspring");
        assert_eq!(offspring.genome_size(), genome_len);
        assert_eq!(offspring.genome, original_genome);
        assert!(org.child_genome.is_none());
        assert_eq!(org.child_copy_progress, 0);
    }

    #[test]
    fn test_organism_new_initializes_correctly() {
        let genome = vec![Instruction::NopA; 10];
        let org = Organism::new(genome);
        assert_eq!(org.genome.len(), 10);
        assert_eq!(org.merit, 1.0);
        assert_eq!(org.instruction_count, 0);
        assert_eq!(org.generation, 0);
        assert_eq!(org.offspring_count, 0);
        assert_eq!(org.tasks_completed, 0);
        assert!(org.child_genome.is_none());
        assert_eq!(org.child_copy_progress, 0);
        assert_eq!(org.gestation_cycles, 0);
    }

    #[test]
    fn test_advance_ip() {
        let mut org = Organism::new(vec![Instruction::NopA; 5]);
        assert_eq!(org.cpu.ip, 0);
        org.advance_ip();
        assert_eq!(org.cpu.ip, 1);
        org.cpu.ip = 4;
        org.advance_ip();
        assert_eq!(org.cpu.ip, 0); // Wraps around
    }

    #[test]
    fn test_current_instruction() {
        let genome = vec![Instruction::NopA, Instruction::NopB, Instruction::Add];
        let mut org = Organism::new(genome);
        assert_eq!(org.current_instruction(), Some(Instruction::NopA));
        org.cpu.ip = 1;
        assert_eq!(org.current_instruction(), Some(Instruction::NopB));
        org.cpu.ip = 2;
        assert_eq!(org.current_instruction(), Some(Instruction::Add));
    }

    #[test]
    fn test_copy_instruction_without_allocation_fails() {
        let mut org = Organism::ancestor();
        let result = org.copy_instruction(0.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_copy_instruction_advances_heads() {
        let mut org = Organism::ancestor();
        org.allocate_child();
        let initial_rh = org.cpu.read_head;
        let initial_wh = org.cpu.write_head;

        org.copy_instruction(0.0);

        assert_eq!(org.cpu.read_head, initial_rh + 1);
        assert_eq!(org.cpu.write_head, initial_wh + 1);
    }

    #[test]
    fn test_copy_instruction_updates_progress() {
        let mut org = Organism::ancestor();
        org.allocate_child();

        assert_eq!(org.child_copy_progress, 0);
        org.copy_instruction(0.0);
        assert_eq!(org.child_copy_progress, 1);
        org.copy_instruction(0.0);
        assert_eq!(org.child_copy_progress, 2);
    }

    #[test]
    fn test_divide_without_child_fails() {
        let mut org = Organism::ancestor();
        let result = org.divide(0.0, 0.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_divide_with_insufficient_copying_fails() {
        let mut org = Organism::ancestor();
        org.allocate_child();

        // Copy less than half
        for _ in 0..20 {
            org.copy_instruction(0.0);
        }

        let result = org.divide(0.0, 0.0);
        assert!(result.is_none());
        // Child should be retained for retry
        assert!(org.child_genome.is_some());
    }

    #[test]
    fn test_divide_resets_parent_state() {
        let mut org = Organism::ancestor();
        org.allocate_child();

        for _ in 0..50 {
            org.copy_instruction(0.0);
        }

        let offspring_count_before = org.offspring_count;
        let result = org.divide(0.0, 0.0);
        assert!(result.is_some());

        assert_eq!(org.offspring_count, offspring_count_before + 1);
        assert_eq!(org.gestation_cycles, 0);
        assert_eq!(org.cpu.ip, 0); // Reset to start
        assert_eq!(org.cpu.read_head, 0);
        assert_eq!(org.cpu.write_head, 0);
        assert_eq!(org.child_copy_progress, 0);
        assert!(org.child_genome.is_none());
    }

    #[test]
    fn test_offspring_generation_increments() {
        let mut org = Organism::ancestor();
        org.allocate_child();

        for _ in 0..50 {
            org.copy_instruction(0.0);
        }

        let offspring = org.divide(0.0, 0.0).unwrap();
        assert_eq!(offspring.generation, org.generation + 1);
    }

    #[test]
    fn test_offspring_starts_with_base_merit() {
        let mut org = Organism::ancestor();
        org.merit = 100.0; // High merit from tasks
        org.allocate_child();

        for _ in 0..50 {
            org.copy_instruction(0.0);
        }

        let offspring = org.divide(0.0, 0.0).unwrap();
        assert_eq!(offspring.merit, 1.0); // Reset to base, not inherited
    }

    #[test]
    fn test_task_completion_flags() {
        let mut org = Organism::ancestor();

        for task_idx in 0..9 {
            assert!(!org.has_completed_task(task_idx));
            org.complete_task(task_idx);
            assert!(org.has_completed_task(task_idx));
        }

        // Verify all tasks are marked
        for task_idx in 0..9 {
            assert!(org.has_completed_task(task_idx));
        }
    }

    #[test]
    fn test_task_completion_out_of_range() {
        let mut org = Organism::ancestor();
        org.complete_task(10); // Out of range
        assert!(!org.has_completed_task(10));
    }

    #[test]
    fn test_genome_string_conversion() {
        let genome = vec![
            Instruction::HAlloc,
            Instruction::HCopy,
            Instruction::HDivide,
        ];
        let org = Organism::new(genome);
        assert_eq!(org.genome_string(), "rts");
    }

    #[test]
    fn test_genome_size() {
        let org = Organism::new(vec![Instruction::NopA; 25]);
        assert_eq!(org.genome_size(), 25);
    }

    #[test]
    fn test_allocate_child_twice() {
        let mut org = Organism::ancestor();
        org.allocate_child();
        assert!(org.child_genome.is_some());

        // Second allocation should warn but work
        org.allocate_child();
        assert!(org.child_genome.is_some());
    }

    #[test]
    fn test_copy_with_mutation() {
        let mut org = Organism::ancestor();
        org.allocate_child();

        let mut mutation_occurred = false;
        for _ in 0..100 {
            let original_inst = org.genome[org.cpu.read_head];
            org.copy_instruction(1.0); // 100% mutation rate

            if org.child_genome.is_some() {
                let copied_inst = org.child_genome.as_ref().unwrap()[org.cpu.write_head - 1];
                if copied_inst != original_inst {
                    mutation_occurred = true;
                    break;
                }
            }
        }
        assert!(
            mutation_occurred,
            "With 100% mutation rate, at least one mutation should occur"
        );
    }

    #[test]
    fn test_divide_with_insertions() {
        let mut org = Organism::ancestor();
        org.allocate_child();

        for _ in 0..50 {
            org.copy_instruction(0.0);
        }

        let offspring = org.divide(1.0, 0.0).unwrap(); // 100% insertion rate
        assert_eq!(offspring.genome_size(), 51);
    }

    #[test]
    fn test_divide_with_deletions() {
        let mut org = Organism::ancestor();
        org.allocate_child();

        for _ in 0..50 {
            org.copy_instruction(0.0);
        }

        let offspring = org.divide(0.0, 1.0).unwrap(); // 100% deletion rate
        assert_eq!(offspring.genome_size(), 49);
    }

    #[test]
    fn test_empty_genome_after_deletions_gets_nop() {
        let mut org = Organism::new(vec![Instruction::NopA; 1]);
        org.allocate_child();

        for _ in 0..1 {
            org.copy_instruction(0.0);
        }

        let offspring = org.divide(0.0, 1.0).unwrap(); // 100% deletion rate
        assert_eq!(offspring.genome_size(), 1);
        assert_eq!(offspring.genome[0], Instruction::NopC);
    }

    #[test]
    fn test_divide_with_both_indels() {
        let mut org = Organism::ancestor();
        org.allocate_child();

        for _ in 0..50 {
            org.copy_instruction(0.0);
        }

        let offspring = org.divide(1.0, 1.0).unwrap();
        assert_eq!(offspring.genome_size(), 50);
    }

    #[test]
    fn test_divide_with_low_indel_rates() {
        let mut unchanged_count = 0;
        let trials = 100;

        for _ in 0..trials {
            let mut org = Organism::ancestor();
            org.allocate_child();

            for _ in 0..50 {
                org.copy_instruction(0.0);
            }

            let offspring = org.divide(0.05, 0.05).unwrap();
            if offspring.genome_size() == 50 {
                unchanged_count += 1;
            }
        }

        assert!(
            unchanged_count >= 75,
            "Expected most genomes unchanged with 0.05 rates, got {}/100",
            unchanged_count
        );
    }

    #[test]
    fn test_execute_instruction_increments_counters() {
        let mut org = Organism::new(vec![Instruction::NopA]);
        let initial_count = org.instruction_count;
        let initial_gestation = org.gestation_cycles;

        org.execute_instruction();

        assert_eq!(org.instruction_count, initial_count + 1);
        assert_eq!(org.gestation_cycles, initial_gestation + 1);
    }

    #[test]
    fn test_organism_clone() {
        let org = Organism::ancestor();
        let cloned = org.clone();

        assert_eq!(org.genome.len(), cloned.genome.len());
        assert_eq!(org.merit, cloned.merit);
        assert_eq!(org.generation, cloned.generation);
    }

    #[test]
    fn test_fitness_calculation() {
        let mut org = Organism::ancestor();
        org.merit = 2.0;
        org.gestation_cycles = 100;

        // Fitness = merit / gestation_cycles
        assert_eq!(org.fitness(), 0.02);
    }

    #[test]
    fn test_fitness_with_high_merit() {
        let mut org = Organism::ancestor();
        org.merit = 16.0; // As if completed XOR task
        org.gestation_cycles = 100;

        assert_eq!(org.fitness(), 0.16);
    }

    #[test]
    fn test_fitness_comparison() {
        let mut org1 = Organism::ancestor();
        org1.merit = 2.0;
        org1.gestation_cycles = 100;

        let mut org2 = Organism::ancestor();
        org2.merit = 1.0;
        org2.gestation_cycles = 100;

        assert!(org1.fitness() > org2.fitness());
    }

    #[test]
    fn test_fitness_after_task_completion() {
        let mut org = Organism::ancestor();
        org.merit = 1.0;
        org.gestation_cycles = 100;
        let fitness_before = org.fitness();

        // Simulate completing NOT task (2x merit bonus)
        org.merit *= 2.0;
        let fitness_after = org.fitness();

        assert_eq!(fitness_after, fitness_before * 2.0);
    }

    #[test]
    fn test_age() {
        let mut org = Organism::ancestor();
        assert_eq!(org.age(), 0);
        org.execute_instruction();
        assert_eq!(org.age(), 1);
        org.execute_instruction();
        assert_eq!(org.age(), 2);
    }

    #[test]
    fn test_gestation_cycles_reset_after_divide() {
        let mut org = Organism::ancestor();
        org.allocate_child();

        // Simulate some gestation cycles
        for _ in 0..10 {
            org.execute_instruction();
        }
        assert_eq!(org.gestation_cycles, 10);

        // Complete copying
        for _ in 0..50 {
            org.copy_instruction(0.0);
        }

        // Divide should reset gestation_cycles
        let _offspring = org.divide(0.0, 0.0).unwrap();
        assert_eq!(org.gestation_cycles, 0);
    }
}
