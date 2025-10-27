use crate::tasks::Task;
use crate::world::World;
use egui::{Color32, Rect, Vec2};

/// Main application state
pub struct AvidaApp {
    pub world: World,
    pub paused: bool,
    pub updates_per_frame: u32,
    pub selected_cell: Option<(usize, usize)>,
    pub show_inspector: bool,
    pub color_mode: ColorMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    Age,
    Merit,
    Fitness,
    GenomeSize,
    Tasks,
}

impl AvidaApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut world = World::new();
        world.inject_ancestor();

        Self {
            world,
            paused: false,
            updates_per_frame: 1,
            selected_cell: None,
            show_inspector: true,
            color_mode: ColorMode::Tasks,
        }
    }
}

impl eframe::App for AvidaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Continuously repaint
        ctx.request_repaint();

        // Run simulation updates
        if !self.paused {
            for _ in 0..self.updates_per_frame {
                self.world.update();
            }
        }

        // Top panel with controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Avida-RS - Digital Evolution Simulation");

                ui.separator();

                if ui
                    .button(if self.paused { "▶ Play" } else { "⏸ Pause" })
                    .clicked()
                {
                    self.paused = !self.paused;
                }

                ui.separator();

                ui.label("Speed:");
                ui.add(egui::Slider::new(&mut self.updates_per_frame, 1..=100).logarithmic(true));

                ui.separator();

                if ui.button("Reset").clicked() {
                    self.world.clear();
                    self.world.inject_ancestor();
                }

                if ui.button("Reset (Tasks)").clicked() {
                    self.world.clear();
                    self.world.inject_ancestor_with_tasks();
                }

                ui.separator();

                ui.label(format!("Updates: {}", self.world.total_updates));
                ui.label(format!("Pop: {}", self.world.population_size));
            });
        });

        // Left panel with statistics
        egui::SidePanel::left("stats_panel")
            .min_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Statistics");
                ui.separator();

                ui.label(format!("Population: {}", self.world.population_size));
                ui.label(format!("Total Births: {}", self.world.total_births));
                ui.label(format!("Total Deaths: {}", self.world.total_deaths));
                ui.label(format!("Updates: {}", self.world.total_updates));

                ui.add_space(10.0);
                ui.label(format!(
                    "Avg Genome Size: {:.1}",
                    self.world.average_genome_size()
                ));
                ui.label(format!("Avg Merit: {:.2}", self.world.average_merit()));
                ui.label(format!("Avg Fitness: {:.4}", self.world.average_fitness()));

                ui.add_space(10.0);
                ui.separator();
                ui.heading("Mutation Rates");

                ui.horizontal(|ui| {
                    ui.label("Copy:");
                    ui.add(
                        egui::DragValue::new(&mut self.world.copy_mutation_rate)
                            .speed(0.0001)
                            .range(0.0..=1.0),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Insertion:");
                    ui.add(
                        egui::DragValue::new(&mut self.world.insertion_rate)
                            .speed(0.001)
                            .range(0.0..=1.0),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Deletion:");
                    ui.add(
                        egui::DragValue::new(&mut self.world.deletion_rate)
                            .speed(0.001)
                            .range(0.0..=1.0),
                    );
                });

                ui.add_space(10.0);
                ui.separator();
                ui.heading("Tasks Completed");

                let task_stats = self.world.task_statistics();
                for task in Task::all() {
                    let count = task_stats[task as usize];
                    ui.label(format!(
                        "{}: {} ({}x merit)",
                        task.name(),
                        count,
                        task.merit_multiplier() as i32
                    ));
                }

                ui.add_space(10.0);
                ui.separator();
                ui.heading("Display Mode");

                ui.radio_value(&mut self.color_mode, ColorMode::Age, "Age");
                ui.radio_value(&mut self.color_mode, ColorMode::Merit, "Merit");
                ui.radio_value(&mut self.color_mode, ColorMode::Fitness, "Fitness");
                ui.radio_value(&mut self.color_mode, ColorMode::GenomeSize, "Genome Size");
                ui.radio_value(&mut self.color_mode, ColorMode::Tasks, "Tasks Completed");

                ui.add_space(10.0);
                ui.checkbox(&mut self.show_inspector, "Show Inspector");
            });

        // Right panel with organism inspector
        if self.show_inspector {
            egui::SidePanel::right("inspector_panel")
                .min_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Organism Inspector");
                    ui.separator();

                    if let Some((x, y)) = self.selected_cell {
                        if let Some(org) = self.world.get_organism(x, y) {
                            ui.label(format!("Position: ({}, {})", x, y));
                            ui.label(format!("Generation: {}", org.generation));
                            ui.label(format!("Age: {}", org.age()));
                            ui.label(format!("Merit: {:.2}", org.merit));
                            ui.label(format!("Gestation Cycles: {}", org.gestation_cycles));
                            ui.label(format!("Fitness: {:.4}", org.fitness()));
                            ui.label(format!("Offspring: {}", org.offspring_count));

                            ui.add_space(10.0);
                            ui.label(format!("Genome Size: {}", org.genome_size()));

                            ui.add_space(10.0);
                            ui.label("Genome:");
                            ui.separator();

                            // Display genome in chunks of 50 characters
                            let genome_str = org.genome_string();
                            for (i, chunk) in genome_str
                                .chars()
                                .collect::<Vec<_>>()
                                .chunks(50)
                                .enumerate()
                            {
                                let chunk_str: String = chunk.iter().collect();
                                ui.label(format!("{:3}: {}", i * 50, chunk_str));
                            }

                            ui.add_space(10.0);
                            ui.label("CPU State:");
                            ui.separator();
                            ui.label(format!("IP: {}", org.cpu.ip));
                            ui.label(format!("AX: {}", org.cpu.registers[0]));
                            ui.label(format!("BX: {}", org.cpu.registers[1]));
                            ui.label(format!("CX: {}", org.cpu.registers[2]));
                            ui.label(format!("Read-Head: {}", org.cpu.read_head));
                            ui.label(format!("Write-Head: {}", org.cpu.write_head));
                            ui.label(format!("Flow-Head: {}", org.cpu.flow_head));

                            ui.add_space(10.0);
                            ui.label("Tasks:");
                            ui.separator();
                            for task in Task::all() {
                                if org.has_completed_task(task as u8) {
                                    ui.label(format!("✓ {}", task.name()));
                                }
                            }
                        } else {
                            ui.label("Empty cell");
                        }
                    } else {
                        ui.label("Click on a cell to inspect");
                    }
                });
        }

        // Central panel with world grid
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            let (width, height) = self.world.dimensions();

            let cell_size =
                (available_size.x.min(available_size.y) / width.max(height) as f32).max(2.0);

            let (response, painter) = ui.allocate_painter(
                Vec2::new(cell_size * width as f32, cell_size * height as f32),
                egui::Sense::click(),
            );

            // Handle click selection
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let click_pos = pos - response.rect.min;
                    let grid_x = (click_pos.x / cell_size) as usize;
                    let grid_y = (click_pos.y / cell_size) as usize;

                    if grid_x < width && grid_y < height {
                        self.selected_cell = Some((grid_x, grid_y));
                    }
                }
            }

            // Draw grid
            for y in 0..height {
                for x in 0..width {
                    let rect = Rect::from_min_size(
                        response.rect.min + Vec2::new(x as f32 * cell_size, y as f32 * cell_size),
                        Vec2::new(cell_size, cell_size),
                    );

                    let color = if let Some(org) = self.world.get_organism(x, y) {
                        match self.color_mode {
                            ColorMode::Age => {
                                let intensity = ((org.age() % 1000) as f32 / 1000.0 * 255.0) as u8;
                                Color32::from_rgb(intensity, 0, 255 - intensity)
                            }
                            ColorMode::Merit => {
                                let intensity =
                                    (org.merit.log2().clamp(0.0, 8.0) / 8.0 * 255.0) as u8;
                                Color32::from_rgb(intensity, intensity, 0)
                            }
                            ColorMode::Fitness => {
                                // Fitness visualization: higher fitness = brighter
                                // Log scale to handle wide range of fitness values
                                let fitness = org.fitness();
                                let intensity = if fitness > 0.0 {
                                    ((fitness * 100.0).log10().clamp(-2.0, 2.0) + 2.0) / 4.0 * 255.0
                                } else {
                                    0.0
                                } as u8;
                                Color32::from_rgb(intensity, 0, intensity) // Purple: high fitness
                            }
                            ColorMode::GenomeSize => {
                                let size_diff = org.genome_size() as i32 - 50;
                                if size_diff > 0 {
                                    let intensity = (size_diff.min(50) as f32 / 50.0 * 255.0) as u8;
                                    Color32::from_rgb(intensity, 0, 0)
                                } else {
                                    let intensity =
                                        ((-size_diff).min(25) as f32 / 25.0 * 255.0) as u8;
                                    Color32::from_rgb(0, 0, intensity)
                                }
                            }
                            ColorMode::Tasks => {
                                let task_count =
                                    (0..9).filter(|&i| org.has_completed_task(i)).count();
                                let intensity = (task_count as f32 / 9.0 * 255.0) as u8;
                                Color32::from_rgb(0, intensity, 0)
                            }
                        }
                    } else {
                        Color32::from_rgb(20, 20, 20) // Empty cell
                    };

                    painter.rect_filled(rect, 0.0, color);

                    // Highlight selected cell
                    if Some((x, y)) == self.selected_cell {
                        painter.rect_stroke(rect, 0.0, (2.0, Color32::WHITE));
                    }
                }
            }
        });
    }
}
