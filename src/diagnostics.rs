/// Diagnostic tools for analyzing evolutionary dynamics

use crate::world::World;
use crate::instruction::Instruction;
use std::collections::HashMap;

/// Analyze the population's instruction diversity
pub fn analyze_instruction_diversity(world: &World) -> HashMap<char, usize> {
    let mut instruction_counts = HashMap::new();

    for cell in &world.grid {
        if let Some(org) = cell {
            for inst in &org.genome {
                let c = inst.to_char();
                *instruction_counts.entry(c).or_insert(0) += 1;
            }
        }
    }

    instruction_counts
}

/// Analyze task-performing potential of the population
pub fn analyze_task_potential(world: &World) -> TaskPotentialStats {
    let mut stats = TaskPotentialStats::default();

    for cell in &world.grid {
        if let Some(org) = cell {
            stats.total_organisms += 1;

            // Count organisms with I/O capability
            if org.genome.iter().any(|i| matches!(i, Instruction::IO)) {
                stats.organisms_with_io += 1;
            }

            // Count organisms with arithmetic
            let has_arithmetic = org.genome.iter().any(|i| {
                matches!(i,
                    Instruction::Add |
                    Instruction::Sub |
                    Instruction::Nand
                )
            });
            if has_arithmetic {
                stats.organisms_with_arithmetic += 1;
            }

            // Count organisms with both I/O and arithmetic
            if org.genome.iter().any(|i| matches!(i, Instruction::IO))
                && has_arithmetic {
                stats.organisms_with_both += 1;
            }

            // Count IO instructions per organism
            let io_count = org.genome.iter()
                .filter(|i| matches!(i, Instruction::IO))
                .count();
            stats.total_io_instructions += io_count;

            // Track tasks completed
            for task_idx in 0..9 {
                if org.has_completed_task(task_idx) {
                    stats.task_completion_counts[task_idx as usize] += 1;
                }
            }

            // Track merit distribution
            stats.merit_sum += org.merit;
            if org.merit > stats.max_merit {
                stats.max_merit = org.merit;
            }
            if org.merit < stats.min_merit {
                stats.min_merit = org.merit;
            }
        }
    }

    stats
}

#[derive(Debug, Default)]
pub struct TaskPotentialStats {
    pub total_organisms: usize,
    pub organisms_with_io: usize,
    pub organisms_with_arithmetic: usize,
    pub organisms_with_both: usize,
    pub total_io_instructions: usize,
    pub task_completion_counts: [usize; 9],
    pub merit_sum: f64,
    pub max_merit: f64,
    pub min_merit: f64,
}

impl TaskPotentialStats {
    pub fn print_report(&self) {
        println!("\n=== Population Task Potential Analysis ===");
        println!("Total organisms: {}", self.total_organisms);
        println!("Organisms with I/O: {} ({:.1}%)",
            self.organisms_with_io,
            100.0 * self.organisms_with_io as f64 / self.total_organisms as f64
        );
        println!("Organisms with arithmetic: {} ({:.1}%)",
            self.organisms_with_arithmetic,
            100.0 * self.organisms_with_arithmetic as f64 / self.total_organisms as f64
        );
        println!("Organisms with both I/O and arithmetic: {} ({:.1}%)",
            self.organisms_with_both,
            100.0 * self.organisms_with_both as f64 / self.total_organisms as f64
        );
        println!("Average I/O instructions per organism: {:.2}",
            self.total_io_instructions as f64 / self.total_organisms as f64
        );
        println!("\nTask Completions:");
        let task_names = ["NOT", "NAND", "AND", "ORN", "OR", "ANDN", "NOR", "XOR", "EQU"];
        for (i, count) in self.task_completion_counts.iter().enumerate() {
            if *count > 0 {
                println!("  {}: {} organisms ({:.1}%)",
                    task_names[i],
                    count,
                    100.0 * *count as f64 / self.total_organisms as f64
                );
            }
        }
        println!("\nMerit Statistics:");
        println!("  Average: {:.2}", self.merit_sum / self.total_organisms as f64);
        println!("  Min: {:.2}", self.min_merit);
        println!("  Max: {:.2}", self.max_merit);
    }
}

/// Sample genomes from the population
pub fn sample_genomes(world: &World, count: usize) -> Vec<String> {
    let mut genomes = Vec::new();
    let mut sampled = 0;

    for cell in &world.grid {
        if let Some(org) = cell {
            if sampled < count {
                genomes.push(org.genome_string());
                sampled += 1;
            } else {
                break;
            }
        }
    }

    genomes
}
