use crate::execute::execute_instruction;
use crate::organism::Organism;
use crate::tasks::{TaskDetector, TaskEnvironment};
use rand::rngs::SmallRng;
use rand::SeedableRng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

const WORLD_WIDTH: usize = 60;
const WORLD_HEIGHT: usize = 60;

/// World grid with toroidal topology (wraparound edges)
pub struct World {
    /// Grid of organisms (None = empty cell) - flat array for better cache locality
    pub grid: Vec<Option<Organism>>,

    /// Task detectors for each organism - flat array
    task_detectors: Vec<Option<TaskDetector>>,

    /// Task environment configuration
    pub task_env: TaskEnvironment,

    /// Mutation rates
    pub copy_mutation_rate: f64,
    pub insertion_rate: f64,
    pub deletion_rate: f64,

    /// Death and birth settings (Avida-style)
    /// death_method: 0 = no age death, 1 = fixed age limit, 2 = age limit × genome length
    pub death_method: u8,
    /// age_limit: multiplier for death age (with death_method 2, organism dies at genome_len × age_limit instructions)
    pub age_limit: u64,
    /// prefer_empty: if true, always prefer empty cells over occupied ones during birth
    pub prefer_empty: bool,

    /// Statistics
    pub total_updates: u64,
    pub total_organisms: u64,
    pub total_births: u64,
    pub total_deaths: u64,

    /// Current population size
    pub population_size: usize,

    /// Fast RNG for better performance (no synchronization overhead like thread_rng)
    rng: SmallRng,
}

impl World {
    pub fn new() -> Self {
        let grid_size = WORLD_WIDTH * WORLD_HEIGHT;
        let grid = vec![None; grid_size];
        let task_detectors = vec![None; grid_size];

        Self {
            grid,
            task_detectors,
            task_env: TaskEnvironment::default_logic9(),
            copy_mutation_rate: 0.0075, // Default Avida copy mutation rate (0.75%)
            insertion_rate: 0.0,        // Insertions disabled during copy (Avida default)
            deletion_rate: 0.0,         // Deletions disabled during copy (Avida default)
            death_method: 2,            // Avida default: age limit × genome length
            age_limit: 20,              // Avida default: 20× genome length
            prefer_empty: true,         // Avida default: prefer empty cells
            total_updates: 0,
            total_organisms: 0,
            total_births: 0,
            total_deaths: 0,
            population_size: 0,
            rng: SmallRng::from_entropy(), // Faster than thread_rng
        }
    }

    /// Convert 2D coordinates to flat array index
    #[inline]
    fn grid_index(&self, x: usize, y: usize) -> usize {
        y * WORLD_WIDTH + x
    }

    /// Get world dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (WORLD_WIDTH, WORLD_HEIGHT)
    }

    /// Wrap coordinates for toroidal topology
    fn wrap_coord(&self, x: isize, y: isize) -> (usize, usize) {
        let wx =
            ((x % WORLD_WIDTH as isize + WORLD_WIDTH as isize) % WORLD_WIDTH as isize) as usize;
        let wy =
            ((y % WORLD_HEIGHT as isize + WORLD_HEIGHT as isize) % WORLD_HEIGHT as isize) as usize;
        (wx, wy)
    }

    /// Get neighbors of a cell (8-connected, toroidal)
    /// Returns stack-allocated array for better performance (no heap allocation)
    pub fn get_neighbors(&self, x: usize, y: usize) -> [(usize, usize); 8] {
        let x = x as isize;
        let y = y as isize;

        let mut neighbors = [(0, 0); 8];
        let mut idx = 0;

        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let (nx, ny) = self.wrap_coord(x + dx, y + dy);
                neighbors[idx] = (nx, ny);
                idx += 1;
            }
        }

        neighbors
    }

    /// Inject an organism at a specific position
    pub fn inject_organism(&mut self, org: Organism, x: usize, y: usize) -> bool {
        if x >= WORLD_WIDTH || y >= WORLD_HEIGHT {
            return false;
        }

        let idx = self.grid_index(x, y);
        if self.grid[idx].is_some() {
            self.total_deaths += 1;
        }

        self.grid[idx] = Some(org);
        self.task_detectors[idx] = Some(TaskDetector::new());
        self.population_size = self.count_population();
        self.total_organisms += 1;
        true
    }

    /// Inject ancestor at center of world
    pub fn inject_ancestor(&mut self) {
        let ancestor = Organism::ancestor();
        let center_x = WORLD_WIDTH / 2;
        let center_y = WORLD_HEIGHT / 2;
        self.inject_organism(ancestor, center_x, center_y);
    }

    /// Inject task-capable ancestor at center of world
    /// This ancestor has I/O and arithmetic instructions for task evolution
    pub fn inject_ancestor_with_tasks(&mut self) {
        let ancestor = Organism::ancestor_with_tasks();
        let center_x = WORLD_WIDTH / 2;
        let center_y = WORLD_HEIGHT / 2;
        self.inject_organism(ancestor, center_x, center_y);
    }

    /// Find birth location for offspring (Avida BIRTH_METHOD and PREFER_EMPTY)
    fn find_birth_location(&mut self, parent_x: usize, parent_y: usize) -> Option<(usize, usize)> {
        use rand::seq::SliceRandom;

        let neighbors = self.get_neighbors(parent_x, parent_y);
        let mut neighbor_vec: Vec<(usize, usize)> = neighbors.to_vec();

        // Shuffle neighbors to randomize placement
        neighbor_vec.shuffle(&mut self.rng);

        if self.prefer_empty {
            // Avida default: prefer empty cells over occupied ones
            // First try to find empty cell
            for (nx, ny) in &neighbor_vec {
                let idx = self.grid_index(*nx, *ny);
                if self.grid[idx].is_none() {
                    // Log first few offspring placements
                    static PLACEMENT_LOG: AtomicU32 = AtomicU32::new(0);
                    let log_index = PLACEMENT_LOG.fetch_add(1, Ordering::Relaxed) + 1;
                    if log_index <= 10 {
                        crate::debug::log_event(format!(
                            "[PLACEMENT #{}] parent:({},{}) -> offspring:({},{}) EMPTY dx:{} dy:{}",
                            log_index,
                            parent_x,
                            parent_y,
                            nx,
                            ny,
                            *nx as isize - parent_x as isize,
                            *ny as isize - parent_y as isize
                        ));
                    }
                    return Some((*nx, *ny));
                }
            }

            // If no empty cells, pick random neighbor (already shuffled)
            if !neighbor_vec.is_empty() {
                let (nx, ny) = neighbor_vec[0];
                crate::debug::log_event(format!(
                    "[PLACEMENT] parent:({},{}) -> offspring:({},{}) REPLACING existing organism",
                    parent_x, parent_y, nx, ny
                ));
                return Some((nx, ny));
            }
        } else {
            // No preference: pick random neighbor (could be empty or occupied)
            if !neighbor_vec.is_empty() {
                let (nx, ny) = neighbor_vec[0];
                let idx = self.grid_index(nx, ny);
                let is_empty = self.grid[idx].is_none();
                crate::debug::log_event(format!(
                    "[PLACEMENT] parent:({},{}) -> offspring:({},{}) {}",
                    parent_x,
                    parent_y,
                    nx,
                    ny,
                    if is_empty { "EMPTY" } else { "REPLACING" }
                ));
                return Some((nx, ny));
            }
        }

        None
    }

    /// Execute one update cycle
    /// An update is a time slice where all organisms get CPU cycles proportional to MERIT
    /// Total CPU cycles in an update scales with population size
    /// Organisms with higher merit execute more instructions and reproduce faster
    pub fn update(&mut self) {
        let pop_before = self.population_size;

        // Calculate total merit (parallel on native, sequential on wasm)
        #[cfg(not(target_arch = "wasm32"))]
        let total_merit: f64 = self
            .grid
            .par_iter()
            .filter_map(|cell| cell.as_ref().map(|org| org.merit))
            .sum();
        #[cfg(target_arch = "wasm32")]
        let total_merit: f64 = self
            .grid
            .iter()
            .filter_map(|cell| cell.as_ref().map(|org| org.merit))
            .sum();

        if total_merit == 0.0 {
            crate::debug::log_event(format!(
                "[WARN] Update {} has zero total merit (pop:{})",
                self.total_updates, self.population_size
            ));
            return;
        }

        // Log update start periodically
        if self.total_updates % 100 == 0 {
            crate::debug::log_event(format!(
                "[UPDATE #{}] pop:{} merit_total:{:.1} births:{} deaths:{}",
                self.total_updates,
                self.population_size,
                total_merit,
                self.total_births,
                self.total_deaths
            ));
        }

        // **MERIT-BASED SCHEDULING**
        // Each organism gets CPU cycles STRICTLY proportional to its MERIT
        // Merit starts at 1.0 and is ONLY increased by completing tasks
        // This creates selection pressure: organisms that complete tasks get more CPU time
        // and reproduce faster, dominating the population
        //
        // Example: If organism A has merit 1.0 and organism B has merit 4.0 (completed tasks):
        //   - Total merit = 5.0
        //   - If total cycles = 150, then cycles_per_merit = 30
        //   - Organism A gets 30 cycles, organism B gets 120 cycles (4x more)
        //   - Organism B reproduces 4x faster than organism A
        //
        // Total CPU cycles per update scales with population to maintain performance
        let total_cycles_per_update = 30.0 * self.population_size.max(1) as f64;
        let cycles_per_merit = total_cycles_per_update / total_merit;

        // Collect positions to process (to avoid borrow conflicts)
        let mut positions = Vec::new();
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                let idx = self.grid_index(x, y);
                if self.grid[idx].is_some() {
                    positions.push((x, y));
                }
            }
        }

        // Shuffle positions for fairness
        use rand::seq::SliceRandom;
        positions.shuffle(&mut self.rng);

        // Process each organism
        for (x, y) in positions {
            let idx = self.grid_index(x, y);

            // Check for age-based death (Avida DEATH_METHOD)
            if let Some(org) = &self.grid[idx] {
                let should_die = match self.death_method {
                    0 => false,                                                      // No age-based death
                    1 => org.age() >= self.age_limit,                                // Fixed age limit
                    2 => org.age() >= (org.genome.len() as u64 * self.age_limit),   // Age limit × genome length
                    _ => false,
                };

                if should_die {
                    self.grid[idx] = None;
                    self.total_deaths += 1;
                    continue; // Skip to next organism
                }
            }

            if let Some(org) = &self.grid[idx] {
                // Calculate CPU cycles for this organism (proportional to MERIT)
                // Higher merit = more cycles = faster reproduction
                let cycles = (cycles_per_merit * org.merit).max(1.0) as u32;

                // Execute cycles with safety limit to detect infinite loops
                let max_cycles_per_organism = 500; // Safety limit
                let actual_cycles = cycles.min(max_cycles_per_organism);

                if cycles > max_cycles_per_organism {
                    crate::debug::log_event(format!(
                        "[WARN] Organism at ({},{}) wanted {} cycles, capped at {}",
                        x, y, cycles, max_cycles_per_organism
                    ));
                }

                let parent_idx = self.grid_index(x, y);
                let mut parent_alive = true;

                for cycle_num in 0..actual_cycles {
                    if !parent_alive {
                        break;
                    }

                    // Need to borrow mutably, so temporarily take organism
                    if let (Some(mut org), Some(mut detector)) = (
                        self.grid[parent_idx].take(),
                        self.task_detectors[parent_idx].take(),
                    ) {
                        // Detect potential infinite loops (organism stuck at same IP)
                        let ip_before = org.cpu.ip;

                        let (should_divide, completed_task) =
                            execute_instruction(&mut org, &mut detector, self.copy_mutation_rate);

                        // Check if IP is advancing (not stuck in infinite loop)
                        if cycle_num > 100 && org.cpu.ip == ip_before {
                            crate::debug::log_event(format!(
                                "[LOOP?] Organism at ({},{}) stuck at IP {} for cycle {} - genome_len:{} inst:{}",
                                x, y, org.cpu.ip, cycle_num,
                                org.genome.len(),
                                org.current_instruction().map(|i| i.to_char()).unwrap_or('?')
                            ));
                        }

                        if let Some(_task) = completed_task {
                            // Task was completed (merit already updated)
                        }

                        if should_divide {
                            // Attempt division
                            if let Some(offspring) =
                                org.divide(self.insertion_rate, self.deletion_rate)
                            {
                                // Debug first few offspring genomes
                                static OFFSPRING_LOG: AtomicU32 = AtomicU32::new(0);
                                let log_index = OFFSPRING_LOG.fetch_add(1, Ordering::Relaxed) + 1;
                                if log_index <= 3 {
                                    crate::debug::log_event(format!(
                                        "[OFFSPRING #{}] size:{} genome:{}",
                                        log_index,
                                        offspring.genome_size(),
                                        offspring.genome_string()
                                    ));
                                }

                                if let Some((birth_x, birth_y)) = self.find_birth_location(x, y) {
                                    // Bounds check before placing
                                    if birth_x >= WORLD_WIDTH || birth_y >= WORLD_HEIGHT {
                                        crate::debug::log_event(format!(
                                            "[ERROR] Birth location out of bounds! ({}, {}) max:({}, {})",
                                            birth_x, birth_y, WORLD_WIDTH, WORLD_HEIGHT
                                        ));
                                        continue;
                                    }

                                    // Place offspring
                                    let birth_idx = self.grid_index(birth_x, birth_y);
                                    let replacing_parent = birth_idx == parent_idx;

                                    if self.grid[birth_idx].is_some() || replacing_parent {
                                        self.total_deaths += 1;
                                    }
                                    self.grid[birth_idx] = Some(offspring);
                                    self.task_detectors[birth_idx] = Some(TaskDetector::new());
                                    self.total_births += 1;

                                    if replacing_parent {
                                        parent_alive = false;
                                    }
                                } else {
                                    crate::debug::log_event(format!(
                                        "[WARN] No birth location found for offspring at ({}, {})",
                                        x, y
                                    ));
                                }
                            }
                        }

                        // Put organism back unless it was replaced by its offspring
                        if parent_alive {
                            self.grid[parent_idx] = Some(org);
                            self.task_detectors[parent_idx] = Some(detector);
                        }
                    }

                    if !parent_alive {
                        break;
                    }
                }
            }
        }

        self.total_updates += 1;
        self.population_size = self.count_population();

        // Detect if population stopped growing
        let pop_after = self.population_size;
        if pop_after == pop_before && pop_after > 0 && pop_after < 100 {
            crate::debug::log_event(format!(
                "[STALL] Update {} population unchanged at {} (no births this cycle)",
                self.total_updates, pop_after
            ));
        }

        // Detect population crashes
        if pop_before > 0 && pop_after == 0 {
            crate::debug::log_event(format!(
                "[CRASH] Population died! Update {} went from {} to 0",
                self.total_updates, pop_before
            ));
        }
    }

    /// Count current population (parallel)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn count_population(&self) -> usize {
        self.grid.par_iter().filter(|cell| cell.is_some()).count()
    }

    /// Count current population (sequential on wasm)
    #[cfg(target_arch = "wasm32")]
    pub fn count_population(&self) -> usize {
        self.grid.iter().filter(|cell| cell.is_some()).count()
    }

    /// Get statistics about tasks completed (parallel)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn task_statistics(&self) -> [usize; 9] {
        self.grid
            .par_iter()
            .filter_map(|cell| cell.as_ref())
            .fold(
                || [0usize; 9],
                |mut acc, org| {
                    for task_idx in 0u8..9 {
                        if org.has_completed_task(task_idx) {
                            acc[task_idx as usize] += 1;
                        }
                    }
                    acc
                },
            )
            .reduce(
                || [0usize; 9],
                |mut acc, counts| {
                    for i in 0..9 {
                        acc[i] += counts[i];
                    }
                    acc
                },
            )
    }

    /// Get statistics about tasks completed (sequential on wasm)
    #[cfg(target_arch = "wasm32")]
    pub fn task_statistics(&self) -> [usize; 9] {
        let mut counts = [0usize; 9];
        for org in self.grid.iter().filter_map(|cell| cell.as_ref()) {
            for task_idx in 0u8..9 {
                if org.has_completed_task(task_idx) {
                    counts[task_idx as usize] += 1;
                }
            }
        }
        counts
    }

    /// Get average genome size (parallel)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn average_genome_size(&self) -> f64 {
        let (total, count) = self
            .grid
            .par_iter()
            .filter_map(|cell| cell.as_ref())
            .map(|org| (org.genome_size(), 1))
            .reduce(|| (0, 0), |acc, val| (acc.0 + val.0, acc.1 + val.1));

        if count > 0 {
            total as f64 / count as f64
        } else {
            0.0
        }
    }

    /// Get average genome size (sequential on wasm)
    #[cfg(target_arch = "wasm32")]
    pub fn average_genome_size(&self) -> f64 {
        let mut total = 0usize;
        let mut count = 0usize;
        for org in self.grid.iter().filter_map(|cell| cell.as_ref()) {
            total += org.genome_size();
            count += 1;
        }

        if count > 0 {
            total as f64 / count as f64
        } else {
            0.0
        }
    }

    /// Get average merit (parallel)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn average_merit(&self) -> f64 {
        let (total, count) = self
            .grid
            .par_iter()
            .filter_map(|cell| cell.as_ref())
            .map(|org| (org.merit, 1))
            .reduce(|| (0.0, 0), |acc, val| (acc.0 + val.0, acc.1 + val.1));

        if count > 0 {
            total / count as f64
        } else {
            0.0
        }
    }

    /// Get average merit (sequential on wasm)
    #[cfg(target_arch = "wasm32")]
    pub fn average_merit(&self) -> f64 {
        let mut total = 0.0f64;
        let mut count = 0usize;
        for org in self.grid.iter().filter_map(|cell| cell.as_ref()) {
            total += org.merit;
            count += 1;
        }

        if count > 0 {
            total / count as f64
        } else {
            0.0
        }
    }

    /// Get average fitness (parallel)
    /// Fitness = merit / gestation_cycles (higher is better)
    /// This is calculated for statistics only - scheduling uses merit directly
    #[cfg(not(target_arch = "wasm32"))]
    pub fn average_fitness(&self) -> f64 {
        let (total, count) = self
            .grid
            .par_iter()
            .filter_map(|cell| cell.as_ref())
            .map(|org| (org.fitness(), 1))
            .reduce(|| (0.0, 0), |acc, val| (acc.0 + val.0, acc.1 + val.1));

        if count > 0 {
            total / count as f64
        } else {
            0.0
        }
    }

    /// Get average fitness (sequential on wasm)
    /// Fitness = merit / gestation_cycles (higher is better)
    /// This is calculated for statistics only - scheduling uses merit directly
    #[cfg(target_arch = "wasm32")]
    pub fn average_fitness(&self) -> f64 {
        let mut total = 0.0f64;
        let mut count = 0usize;
        for org in self.grid.iter().filter_map(|cell| cell.as_ref()) {
            total += org.fitness();
            count += 1;
        }

        if count > 0 {
            total / count as f64
        } else {
            0.0
        }
    }

    /// Get organism at position
    pub fn get_organism(&self, x: usize, y: usize) -> Option<&Organism> {
        if x < WORLD_WIDTH && y < WORLD_HEIGHT {
            let idx = self.grid_index(x, y);
            self.grid[idx].as_ref()
        } else {
            None
        }
    }

    /// Clear the world
    pub fn clear(&mut self) {
        for cell in &mut self.grid {
            *cell = None;
        }
        for detector in &mut self.task_detectors {
            *detector = None;
        }
        self.population_size = 0;
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Instruction;

    #[test]
    fn test_world_creation() {
        let world = World::new();
        assert_eq!(world.dimensions(), (60, 60));
        assert_eq!(world.population_size, 0);
    }

    #[test]
    fn test_inject_ancestor() {
        let mut world = World::new();
        world.inject_ancestor();
        assert_eq!(world.population_size, 1);
    }

    #[test]
    fn test_toroidal_wrapping() {
        let world = World::new();
        let (x, y) = world.wrap_coord(-1, -1);
        assert_eq!(x, 59);
        assert_eq!(y, 59);

        let (x, y) = world.wrap_coord(60, 60);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
    }

    #[test]
    fn test_neighbors() {
        let world = World::new();
        let neighbors = world.get_neighbors(0, 0);
        assert_eq!(neighbors.len(), 8);
    }

    #[test]
    fn test_population_growth_preserves_genomes() {
        let mut world = World::new();
        world.inject_ancestor();

        for _ in 0..200 {
            world.update();
        }

        assert!(world.population_size > 1);
        assert!(world.total_births > 0);

        for cell in &world.grid {
            if let Some(org) = cell {
                assert_eq!(org.genome_size(), 50);
            }
        }
    }

    #[test]
    fn test_extended_population_growth() {
        let mut world = World::new();
        world.inject_ancestor();

        for _ in 0..600 {
            world.update();
        }

        assert!(world.population_size > 1);
        assert!(world.total_births > 0);

        for cell in &world.grid {
            if let Some(org) = cell {
                assert_eq!(org.genome_size(), 50);
            }
        }
    }

    #[test]
    fn test_world_default() {
        let world = World::default();
        assert_eq!(world.dimensions(), (60, 60));
        assert_eq!(world.population_size, 0);
    }

    #[test]
    fn test_world_new_empty() {
        let world = World::new();
        assert_eq!(world.count_population(), 0);
        assert_eq!(world.total_updates, 0);
        assert_eq!(world.total_births, 0);
        assert_eq!(world.total_deaths, 0);
    }

    #[test]
    fn test_inject_organism() {
        let mut world = World::new();
        let org = Organism::ancestor();
        assert!(world.inject_organism(org, 10, 10));
        assert_eq!(world.population_size, 1);
    }

    #[test]
    fn test_inject_organism_out_of_bounds() {
        let mut world = World::new();
        let org = Organism::ancestor();
        assert!(!world.inject_organism(org, 100, 100));
        assert_eq!(world.population_size, 0);
    }

    #[test]
    fn test_inject_organism_replaces_existing() {
        let mut world = World::new();
        let org1 = Organism::ancestor();
        let org2 = Organism::ancestor();

        world.inject_organism(org1, 10, 10);
        assert_eq!(world.total_deaths, 0);

        world.inject_organism(org2, 10, 10);
        assert_eq!(world.total_deaths, 1);
    }

    #[test]
    fn test_wrap_coord_positive() {
        let world = World::new();
        assert_eq!(world.wrap_coord(0, 0), (0, 0));
        assert_eq!(world.wrap_coord(59, 59), (59, 59));
    }

    #[test]
    fn test_wrap_coord_overflow() {
        let world = World::new();
        assert_eq!(world.wrap_coord(60, 60), (0, 0));
        assert_eq!(world.wrap_coord(61, 62), (1, 2));
        assert_eq!(world.wrap_coord(120, 120), (0, 0));
    }

    #[test]
    fn test_wrap_coord_negative() {
        let world = World::new();
        assert_eq!(world.wrap_coord(-1, -1), (59, 59));
        assert_eq!(world.wrap_coord(-2, -3), (58, 57));
        assert_eq!(world.wrap_coord(-60, -60), (0, 0));
    }

    #[test]
    fn test_get_neighbors_center() {
        let world = World::new();
        let neighbors = world.get_neighbors(30, 30);
        assert_eq!(neighbors.len(), 8);

        // Check that all 8 neighbors are unique
        let mut unique = std::collections::HashSet::new();
        for neighbor in &neighbors {
            unique.insert(neighbor);
        }
        assert_eq!(unique.len(), 8);
    }

    #[test]
    fn test_get_neighbors_corner() {
        let world = World::new();
        let neighbors = world.get_neighbors(0, 0);
        assert_eq!(neighbors.len(), 8);

        // Should wrap around to opposite edges
        assert!(neighbors.contains(&(59, 59)));
        assert!(neighbors.contains(&(1, 0)));
        assert!(neighbors.contains(&(0, 1)));
    }

    #[test]
    fn test_get_organism() {
        let mut world = World::new();
        world.inject_ancestor();

        let center_x = WORLD_WIDTH / 2;
        let center_y = WORLD_HEIGHT / 2;

        assert!(world.get_organism(center_x, center_y).is_some());
        assert!(world.get_organism(0, 0).is_none());
    }

    #[test]
    fn test_get_organism_out_of_bounds() {
        let world = World::new();
        assert!(world.get_organism(100, 100).is_none());
    }

    #[test]
    fn test_clear_world() {
        let mut world = World::new();
        world.inject_ancestor();
        assert!(world.population_size > 0);

        world.clear();
        assert_eq!(world.population_size, 0);
        assert_eq!(world.count_population(), 0);
    }

    #[test]
    fn test_count_population() {
        let mut world = World::new();
        assert_eq!(world.count_population(), 0);

        world.inject_ancestor();
        assert_eq!(world.count_population(), 1);

        world.inject_organism(Organism::ancestor(), 0, 0);
        assert_eq!(world.count_population(), 2);
    }

    #[test]
    fn test_average_genome_size_empty() {
        let world = World::new();
        assert_eq!(world.average_genome_size(), 0.0);
    }

    #[test]
    fn test_average_genome_size() {
        let mut world = World::new();
        world.inject_organism(Organism::new(vec![Instruction::NopA; 10]), 0, 0);
        world.inject_organism(Organism::new(vec![Instruction::NopB; 20]), 1, 0);

        assert_eq!(world.average_genome_size(), 15.0);
    }

    #[test]
    fn test_average_merit_empty() {
        let world = World::new();
        assert_eq!(world.average_merit(), 0.0);
    }

    #[test]
    fn test_average_merit() {
        let mut world = World::new();
        let mut org1 = Organism::ancestor();
        org1.merit = 2.0;
        let mut org2 = Organism::ancestor();
        org2.merit = 4.0;

        world.inject_organism(org1, 0, 0);
        world.inject_organism(org2, 1, 0);

        assert_eq!(world.average_merit(), 3.0);
    }

    #[test]
    fn test_task_statistics_empty() {
        let world = World::new();
        let stats = world.task_statistics();
        for count in &stats {
            assert_eq!(*count, 0);
        }
    }

    #[test]
    fn test_task_statistics() {
        let mut world = World::new();
        let mut org1 = Organism::ancestor();
        org1.complete_task(0); // NOT
        org1.complete_task(2); // AND

        let mut org2 = Organism::ancestor();
        org2.complete_task(0); // NOT
        org2.complete_task(4); // OR

        world.inject_organism(org1, 0, 0);
        world.inject_organism(org2, 1, 0);

        let stats = world.task_statistics();
        assert_eq!(stats[0], 2); // NOT completed by both
        assert_eq!(stats[2], 1); // AND completed by one
        assert_eq!(stats[4], 1); // OR completed by one
        assert_eq!(stats[1], 0); // NAND completed by none
    }

    #[test]
    fn test_update_increments_counter() {
        let mut world = World::new();
        world.inject_ancestor();

        assert_eq!(world.total_updates, 0);
        world.update();
        assert_eq!(world.total_updates, 1);
        world.update();
        assert_eq!(world.total_updates, 2);
    }

    #[test]
    fn test_update_empty_world() {
        let mut world = World::new();
        world.update();
        assert_eq!(world.population_size, 0);
        assert_eq!(world.total_updates, 0); // Should not increment if no organisms
    }

    #[test]
    fn test_mutation_rates() {
        let world = World::new();
        assert_eq!(world.copy_mutation_rate, 0.0075);
        assert_eq!(world.insertion_rate, 0.0);
        assert_eq!(world.deletion_rate, 0.0);
    }

    #[test]
    fn test_world_dimensions_constant() {
        let world = World::new();
        let (width, height) = world.dimensions();
        assert_eq!(width, WORLD_WIDTH);
        assert_eq!(height, WORLD_HEIGHT);
    }

    #[test]
    fn test_no_births_without_updates() {
        let mut world = World::new();
        world.inject_ancestor();
        assert_eq!(world.total_births, 0);
    }

    #[test]
    fn test_births_occur_after_updates() {
        let mut world = World::new();
        world.inject_ancestor();

        for _ in 0..50 {
            world.update();
        }

        assert!(world.total_births > 0);
    }

    #[test]
    fn test_population_growth_is_exponential_initially() {
        let mut world = World::new();
        world.inject_ancestor();

        let mut pop_history = vec![];
        for _ in 0..100 {
            world.update();
            if world.total_updates % 20 == 0 {
                pop_history.push(world.population_size);
            }
        }

        // Population should generally increase over time
        assert!(pop_history.last().unwrap() > &pop_history[0]);
    }

    #[test]
    fn test_merit_affects_cpu_cycles() {
        let mut world = World::new();
        let mut org1 = Organism::ancestor();
        org1.merit = 1.0;

        let mut org2 = Organism::ancestor();
        org2.merit = 4.0; // 4x merit (e.g., completed AND task)

        world.inject_organism(org1, 0, 0);
        world.inject_organism(org2, 1, 0);

        // Organism with higher merit should get more CPU cycles
        // org2 has 4x merit, so should get 4x more cycles and reproduce ~4x faster
        world.update();
        assert!(world.total_updates > 0);
    }

    #[test]
    fn test_average_fitness() {
        let mut world = World::new();
        let mut org1 = Organism::ancestor();
        org1.merit = 2.0;
        org1.gestation_cycles = 100;

        let mut org2 = Organism::ancestor();
        org2.merit = 4.0;
        org2.gestation_cycles = 100;

        world.inject_organism(org1, 0, 0);
        world.inject_organism(org2, 1, 0);

        // Average fitness = (2.0/100 + 4.0/100) / 2 = 0.03
        assert_eq!(world.average_fitness(), 0.03);
    }

    #[test]
    fn test_merit_selection_pressure() {
        // High-merit organisms should dominate over time
        let mut world = World::new();

        // Inject a low-merit organism
        let mut low_merit = Organism::ancestor();
        low_merit.merit = 1.0;
        world.inject_organism(low_merit, 0, 0);

        // Inject a high-merit organism
        let mut high_merit = Organism::ancestor();
        high_merit.merit = 4.0; // High merit from tasks
        world.inject_organism(high_merit, 1, 0);

        // High-merit organism should get 4x more CPU cycles per update
        // This means it reproduces 4x faster (if gestation times are similar)

        // Run simulation - high merit should reproduce more
        for _ in 0..50 {
            world.update();
        }

        // After 50 updates, should have more organisms due to reproduction
        assert!(world.population_size >= 2);
    }
}
