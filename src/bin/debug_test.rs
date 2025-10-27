//! Debug test harness - run simulation for a limited time and print debug output
//! Run with: cargo run --bin debug_test --release

use avida_rs::world::World;

fn main() {
    println!("=== AVIDA-RS DEBUG TEST ===\n");
    println!("Step 1: Initializing debug system...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    // Initialize debug system
    avida_rs::debug::init();
    println!("Step 2: Debug system initialized");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    // Create world and inject ancestor
    println!("Step 3: Creating world...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let mut world = World::new();
    println!(
        "World created: {}x{}",
        world.dimensions().0,
        world.dimensions().1
    );
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    world.inject_ancestor();
    println!("Ancestor injected at center");
    println!("Population: {}\n", world.population_size);

    // Run for 1000 updates to test for freeze
    println!("--- Running 1000 updates ---\n");
    for i in 0..1000 {
        world.update();

        // Print progress every 50 updates
        if (i + 1) % 50 == 0 {
            println!(
                "Update {}: pop={} births={} deaths={} avg_size={:.1} avg_merit={:.1}",
                i + 1,
                world.population_size,
                world.total_births,
                world.total_deaths,
                world.average_genome_size(),
                world.average_merit()
            );
        }
    }

    println!("\n--- Simulation Complete ---\n");
    println!("Final population: {}", world.population_size);
    println!("Total births: {}", world.total_births);
    println!("Total deaths: {}", world.total_deaths);
    println!("Average genome size: {:.1}", world.average_genome_size());
    println!("Average merit: {:.1}", world.average_merit());

    // Print task statistics
    println!("\nTask completions:");
    let task_stats = world.task_statistics();
    let task_names = [
        "NOT", "NAND", "AND", "ORN", "OR", "ANDN", "NOR", "XOR", "EQU",
    ];
    for (i, count) in task_stats.iter().enumerate() {
        if *count > 0 {
            println!("  {}: {}", task_names[i], count);
        }
    }

    println!("\n");
    avida_rs::debug::print_stats();

    // Check if replication happened
    if world.total_births == 0 {
        println!("\n⚠️  WARNING: No births occurred! Organism may not be replicating.");
        println!("Check the debug output above for clues.\n");
    } else {
        println!(
            "\n✓ Replication working! {} births in 100 updates.\n",
            world.total_births
        );
    }
}
