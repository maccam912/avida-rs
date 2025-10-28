use crate::cpu::HeadType;
use crate::instruction::Instruction;
use crate::organism::Organism;
use crate::tasks::{Task, TaskDetector};
use std::sync::atomic::{AtomicU32, Ordering};

/// Track instruction execution for monitoring
static INSTRUCTIONS_EXECUTED: AtomicU32 = AtomicU32::new(0);

/// Execute a single instruction for an organism
/// Returns (should_divide, completed_task)
/// The organism's counters (instruction_count, gestation_cycles) are incremented
pub fn execute_instruction(
    organism: &mut Organism,
    task_detector: &mut TaskDetector,
    copy_mutation_rate: f64,
) -> (bool, Option<Task>) {
    let mut should_divide = false;
    let mut completed_task = None;

    // Track total instructions executed globally
    let inst_count = INSTRUCTIONS_EXECUTED.fetch_add(1, Ordering::Relaxed);
    if inst_count == 0 {
        crate::debug::log_event("[START] First instruction execution");
    }

    // Handle skip flag from conditionals
    if organism.cpu.skip_next {
        organism.cpu.skip_next = false;
        organism.advance_ip();
        organism.execute_instruction(); // Still counts as an instruction executed
        return (false, None);
    }

    // Execute the instruction
    if let Some(inst) = organism.current_instruction() {
        match inst {
            // No-ops (a-c)
            Instruction::NopA | Instruction::NopB | Instruction::NopC => {
                // No operation
            }

            // Conditionals (d-e, y)
            Instruction::IfNEqu => {
                // Get complement label following this instruction
                let template_start = organism
                    .cpu
                    .advance_head(organism.cpu.ip, organism.genome.len());
                let template = organism.cpu.read_template(&organism.genome, template_start);

                // Check if BX equals the template pattern (as a hash/value)
                let bx = organism.cpu.registers[1];
                let template_value = template.len() as i32;

                if bx != template_value {
                    // Execute next instruction
                } else {
                    // Skip next instruction
                    organism.cpu.skip_next = true;
                }
            }

            Instruction::IfLess => {
                // Get the register to compare with (default BX vs CX)
                let reg1_idx = 1; // BX
                let reg2_idx = organism.cpu.get_register_index(&organism.genome, 2); // CX default
                let val1 = organism.cpu.registers[reg1_idx];
                let val2 = organism.cpu.registers[reg2_idx];

                if val1 >= val2 {
                    organism.cpu.skip_next = true;
                }
            }

            Instruction::IfLabel => {
                // Check if the last copied label matches the complement of the following label
                let template_start = organism
                    .cpu
                    .advance_head(organism.cpu.ip, organism.genome.len());
                let template = organism.cpu.read_template(&organism.genome, template_start);
                let template_len = template.len();

                // Create complement
                let complement: Vec<Instruction> = template
                    .iter()
                    .filter_map(|inst| inst.complement_nop())
                    .collect();

                // Check if last_copied_label ENDS WITH the complement (not exact match)
                let matches = if complement.is_empty()
                    || organism.cpu.last_copied_label.len() < complement.len()
                {
                    false
                } else {
                    let start = organism.cpu.last_copied_label.len() - complement.len();
                    organism.cpu.last_copied_label[start..] == complement[..]
                };

                static LABEL_COUNT: AtomicU32 = AtomicU32::new(0);
                let label_count = LABEL_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                if label_count <= 5 {
                    let label_str: String = organism
                        .cpu
                        .last_copied_label
                        .iter()
                        .map(|i| i.to_char())
                        .collect();
                    let comp_str: String = complement.iter().map(|i| i.to_char()).collect();
                    crate::debug::log_event(format!(
                        "[IF-LABEL #{}] last_copied:'{}' template:'{}' match:{} skip:{}",
                        label_count,
                        if label_str.len() > 8 {
                            &label_str[label_str.len() - 8..]
                        } else {
                            &label_str
                        },
                        comp_str,
                        matches,
                        !matches
                    ));
                }

                // Advance IP past the template (template nops should not be executed as instructions)
                for _ in 0..template_len {
                    organism.cpu.ip = organism
                        .cpu
                        .advance_head(organism.cpu.ip, organism.genome.len());
                }

                if !matches {
                    organism.cpu.skip_next = true;
                }
            }

            // Stack operations (f-h)
            Instruction::Pop => {
                let value = organism.cpu.pop();
                let reg_idx = organism.cpu.get_register_index(&organism.genome, 1); // Default BX
                organism.cpu.registers[reg_idx] = value;
            }

            Instruction::Push => {
                let reg_idx = organism.cpu.get_register_index(&organism.genome, 1); // Default BX
                let value = organism.cpu.registers[reg_idx];
                organism.cpu.push(value);
            }

            Instruction::SwapStk => {
                organism.cpu.active_stack = !organism.cpu.active_stack;
            }

            // Register operations (i-m)
            Instruction::Swap => {
                let reg1_idx = 1; // BX
                let reg2_idx = organism.cpu.get_register_index(&organism.genome, 2); // Default CX
                organism.cpu.registers.swap(reg1_idx, reg2_idx);
            }

            Instruction::ShiftR => {
                let reg_idx = organism.cpu.get_register_index(&organism.genome, 1); // Default BX
                organism.cpu.registers[reg_idx] >>= 1;
            }

            Instruction::ShiftL => {
                let reg_idx = organism.cpu.get_register_index(&organism.genome, 1); // Default BX
                organism.cpu.registers[reg_idx] <<= 1;
            }

            Instruction::Inc => {
                let reg_idx = organism.cpu.get_register_index(&organism.genome, 1); // Default BX
                organism.cpu.registers[reg_idx] = organism.cpu.registers[reg_idx].wrapping_add(1);
            }

            Instruction::Dec => {
                let reg_idx = organism.cpu.get_register_index(&organism.genome, 1); // Default BX
                organism.cpu.registers[reg_idx] = organism.cpu.registers[reg_idx].wrapping_sub(1);
            }

            // Arithmetic (n-p)
            Instruction::Add => {
                let result = organism.cpu.registers[1].wrapping_add(organism.cpu.registers[2]);
                organism.cpu.registers[1] = result;
            }

            Instruction::Sub => {
                let result = organism.cpu.registers[1].wrapping_sub(organism.cpu.registers[2]);
                organism.cpu.registers[1] = result;
            }

            Instruction::Nand => {
                let result = !(organism.cpu.registers[1] & organism.cpu.registers[2]);
                organism.cpu.registers[1] = result;
            }

            // I/O (q) - Task detection happens here
            Instruction::IO => {
                // Output BX value
                let output = organism.cpu.registers[1];
                organism.cpu.output_buffer.push(output);

                // Check for task completion
                if let Some((task, _inputs)) = task_detector.check_output(output) {
                    let task_idx = task as u8;
                    if !organism.has_completed_task(task_idx) {
                        organism.complete_task(task_idx);
                        // Apply merit bonus with cap to prevent overflow
                        let multiplier = task.merit_multiplier();
                        organism.merit *= multiplier;
                        organism.merit = organism.merit.min(1000.0); // Cap merit to prevent infinity
                        completed_task = Some(task);

                        crate::debug::log_event(format!(
                            "[TASK] Organism gen:{} completed {:?} - merit: {} -> {} ({}x)",
                            organism.generation,
                            task,
                            organism.merit / multiplier,
                            organism.merit,
                            multiplier
                        ));
                    }
                }

                // Input new value into BX
                if !organism.cpu.input_buffer.is_empty() {
                    let input = organism.cpu.input_buffer.remove(0);
                    organism.cpu.registers[1] = input;
                    task_detector.add_input(input);
                } else {
                    // Generate new random input if buffer is empty
                    use rand::Rng;
                    let input = rand::thread_rng().gen::<i32>();
                    organism.cpu.registers[1] = input;
                    task_detector.add_input(input);
                }
            }

            // Genome management (r-u)
            Instruction::HAlloc => {
                organism.allocate_child();
            }

            Instruction::HDivide => {
                // Check if organism is ready to divide
                if organism.child_genome.is_some()
                    && organism.child_copy_progress >= organism.genome.len()
                {
                    // Division will be handled by the world
                    should_divide = true;
                }
            }

            Instruction::HCopy => {
                let _copied = organism.copy_instruction(copy_mutation_rate);
                // Logging handled in organism.copy_instruction()
            }

            Instruction::HSearch => {
                // Search for complement template
                if let Some((distance, size)) = organism
                    .cpu
                    .search_template(&organism.genome, organism.cpu.ip)
                {
                    organism.cpu.registers[1] = distance; // BX = distance
                    organism.cpu.registers[2] = size as i32; // CX = size

                    // Set flow head to end of found template
                    let template_start = organism
                        .cpu
                        .advance_head(organism.cpu.ip, organism.genome.len());
                    let search_start = organism.cpu.advance_head(
                        template_start
                            + organism
                                .cpu
                                .read_template(&organism.genome, template_start)
                                .len(),
                        organism.genome.len(),
                    );
                    let target_pos = (search_start + distance as usize) % organism.genome.len();
                    organism.cpu.flow_head = (target_pos + size) % organism.genome.len();
                } else {
                    // Not found - set registers to 0
                    organism.cpu.registers[1] = 0;
                    organism.cpu.registers[2] = 0;
                    // Set flow head to next instruction (copy loop start)
                    organism.cpu.flow_head = organism
                        .cpu
                        .advance_head(organism.cpu.ip, organism.genome.len());

                    static SEARCH_COUNT: AtomicU32 = AtomicU32::new(0);
                    let search_count = SEARCH_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                    if search_count <= 2 {
                        crate::debug::log_event(format!(
                            "[H-SEARCH #{}] No template found, flow_head set to {}",
                            search_count, organism.cpu.flow_head
                        ));
                    }
                }
            }

            // Head movement (v-x)
            Instruction::MovHead => {
                // Move IP to flow head
                let head_type = organism.cpu.get_head_from_nop(&organism.genome);
                let old_pos = organism.cpu.get_head(head_type);
                let target = if head_type == HeadType::InstructionPointer {
                    (organism.cpu.flow_head + organism.genome.len() - 1) % organism.genome.len()
                } else {
                    organism.cpu.flow_head
                };
                organism.cpu.set_head(head_type, target);

                static MOV_COUNT: AtomicU32 = AtomicU32::new(0);
                let mov_count = MOV_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                if mov_count <= 5 {
                    crate::debug::log_event(format!(
                        "[MOV-HEAD #{}] {:?} {} -> {} (flow:{})",
                        mov_count, head_type, old_pos, target, organism.cpu.flow_head
                    ));
                }
            }

            Instruction::JmpHead => {
                // Move IP by CX offset
                let offset = organism.cpu.registers[2];
                let head_type = organism.cpu.get_head_from_nop(&organism.genome);
                let current = organism.cpu.get_head(head_type);
                let new_pos = organism
                    .cpu
                    .move_head(current, offset, organism.genome.len());
                organism.cpu.set_head(head_type, new_pos);
            }

            Instruction::GetHead => {
                // Copy head position to CX
                let head_type = organism.cpu.get_head_from_nop(&organism.genome);
                let position = organism.cpu.get_head(head_type);
                organism.cpu.registers[2] = position as i32;
            }

            // Flow control (z)
            Instruction::SetFlow => {
                // Set flow head to position in CX
                let position = organism.cpu.registers[2] as usize % organism.genome.len();
                organism.cpu.flow_head = position;
            }
        }
    }

    // Advance IP and increment organism counters
    organism.advance_ip();
    organism.execute_instruction(); // Increments instruction_count and gestation_cycles

    (should_divide, completed_task)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nop_execution() {
        let mut org = Organism::new(vec![Instruction::NopA]);
        let mut detector = TaskDetector::new();
        let (divide, _) = execute_instruction(&mut org, &mut detector, 0.0);
        assert!(!divide);
        assert_eq!(org.cpu.ip, 0); // Wrapped around
        assert_eq!(org.instruction_count, 1); // Counter incremented
    }

    #[test]
    fn test_add_instruction() {
        let mut org = Organism::new(vec![Instruction::Add]);
        org.cpu.registers[1] = 10; // BX
        org.cpu.registers[2] = 5; // CX

        let mut detector = TaskDetector::new();
        execute_instruction(&mut org, &mut detector, 0.0);

        assert_eq!(org.cpu.registers[1], 15); // BX = 10 + 5
        assert_eq!(org.instruction_count, 1);
    }

    #[test]
    fn test_inc_instruction() {
        let mut org = Organism::new(vec![Instruction::Inc]);
        org.cpu.registers[1] = 42;

        let mut detector = TaskDetector::new();
        execute_instruction(&mut org, &mut detector, 0.0);

        assert_eq!(org.cpu.registers[1], 43);
        assert_eq!(org.instruction_count, 1);
    }

    #[test]
    fn test_push_pop() {
        let mut org = Organism::new(vec![Instruction::Push, Instruction::Pop]);
        org.cpu.registers[1] = 100;

        let mut detector = TaskDetector::new();

        // Push
        org.cpu.ip = 0;
        execute_instruction(&mut org, &mut detector, 0.0);
        assert_eq!(org.cpu.active_stack_ref().len(), 1);
        assert_eq!(org.instruction_count, 1);

        // Pop
        org.cpu.registers[1] = 0; // Clear BX
        execute_instruction(&mut org, &mut detector, 0.0);
        assert_eq!(org.cpu.registers[1], 100);
        assert_eq!(org.instruction_count, 2);
    }

    #[test]
    fn test_instruction_counter_increments() {
        let mut org = Organism::new(vec![
            Instruction::NopA,
            Instruction::NopB,
            Instruction::NopC,
        ]);
        let mut detector = TaskDetector::new();

        assert_eq!(org.instruction_count, 0);
        execute_instruction(&mut org, &mut detector, 0.0);
        assert_eq!(org.instruction_count, 1);
        execute_instruction(&mut org, &mut detector, 0.0);
        assert_eq!(org.instruction_count, 2);
        execute_instruction(&mut org, &mut detector, 0.0);
        assert_eq!(org.instruction_count, 3);
    }

    #[test]
    fn test_gestation_cycles_increment() {
        let mut org = Organism::new(vec![Instruction::NopA]);
        let mut detector = TaskDetector::new();

        assert_eq!(org.gestation_cycles, 0);
        execute_instruction(&mut org, &mut detector, 0.0);
        assert_eq!(org.gestation_cycles, 1);
    }

    #[test]
    fn test_minimal_rts_genome_does_not_signal_divide() {
        use crate::instruction::parse_genome;

        let genome = parse_genome("rts").expect("valid minimal genome");
        let mut org = Organism::new(genome);
        let mut detector = TaskDetector::new();

        for _ in 0..30 {
            let (should_divide, _) = execute_instruction(&mut org, &mut detector, 0.0);
            assert!(org.child_copy_progress < org.genome.len());
            assert!(
                !should_divide,
                "minimal rts genome should not be able to divide"
            );
        }
    }
}
