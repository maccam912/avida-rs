//! Task detection and reward system for Avida organisms
//! Based on the default Logic-9 environment

/// The 9 logic tasks organisms can perform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Task {
    Not = 0,  // NOT - bitwise not
    Nand = 1, // NAND - bitwise nand
    And = 2,  // AND - bitwise and
    Orn = 3,  // ORN - or-not (A or not B)
    Or = 4,   // OR - bitwise or
    Andn = 5, // ANDN - and-not (A and not B)
    Nor = 6,  // NOR - bitwise nor
    Xor = 7,  // XOR - bitwise xor
    Equ = 8,  // EQU - equivalence (not xor)
}

impl Task {
    /// Get the merit bonus value for this task
    /// Merit multiplier = 2^value
    pub fn bonus_value(&self) -> f64 {
        match self {
            Task::Not => 1.0,  // 2^1 = 2x merit
            Task::Nand => 1.0, // 2^1 = 2x merit
            Task::And => 2.0,  // 2^2 = 4x merit
            Task::Orn => 2.0,  // 2^2 = 4x merit
            Task::Or => 3.0,   // 2^3 = 8x merit
            Task::Andn => 3.0, // 2^3 = 8x merit
            Task::Nor => 4.0,  // 2^4 = 16x merit
            Task::Xor => 4.0,  // 2^4 = 16x merit
            Task::Equ => 4.0,  // 2^4 = 16x merit
        }
    }

    /// Get the merit multiplier for this task
    pub fn merit_multiplier(&self) -> f64 {
        2.0_f64.powf(self.bonus_value())
    }

    /// Get task name
    pub fn name(&self) -> &'static str {
        match self {
            Task::Not => "NOT",
            Task::Nand => "NAND",
            Task::And => "AND",
            Task::Orn => "ORN",
            Task::Or => "OR",
            Task::Andn => "ANDN",
            Task::Nor => "NOR",
            Task::Xor => "XOR",
            Task::Equ => "EQU",
        }
    }

    /// All tasks in order
    pub fn all() -> [Task; 9] {
        [
            Task::Not,
            Task::Nand,
            Task::And,
            Task::Orn,
            Task::Or,
            Task::Andn,
            Task::Nor,
            Task::Xor,
            Task::Equ,
        ]
    }
}

/// Task detector - checks if an output matches a task given inputs
#[derive(Debug, Clone)]
pub struct TaskDetector {
    /// Previous inputs for task checking
    inputs: Vec<i32>,
}

impl TaskDetector {
    pub fn new() -> Self {
        Self { inputs: vec![] }
    }

    /// Record an input value
    pub fn add_input(&mut self, value: i32) {
        self.inputs.push(value);
        // Keep only last 3 inputs
        if self.inputs.len() > 3 {
            self.inputs.remove(0);
        }
    }

    /// Check if an output value corresponds to any task
    /// Returns the task if detected, along with the inputs used
    pub fn check_output(&self, output: i32) -> Option<(Task, Vec<i32>)> {
        // Need at least 2 inputs for most tasks
        if self.inputs.len() < 2 {
            return None;
        }

        let a = self.inputs[self.inputs.len() - 2];
        let b = self.inputs[self.inputs.len() - 1];

        // Check each task
        for task in Task::all() {
            let expected = match task {
                Task::Not => !b,
                Task::Nand => !(a & b),
                Task::And => a & b,
                Task::Orn => a | !b,
                Task::Or => a | b,
                Task::Andn => a & !b,
                Task::Nor => !(a | b),
                Task::Xor => a ^ b,
                Task::Equ => !(a ^ b),
            };

            if expected == output {
                return Some((task, vec![a, b]));
            }
        }

        None
    }

    /// Clear input history
    pub fn clear(&mut self) {
        self.inputs.clear();
    }
}

impl Default for TaskDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Task environment configuration
pub struct TaskEnvironment {
    /// Whether each task is enabled
    pub task_enabled: [bool; 9],

    /// Maximum times each task can be rewarded (0 = unlimited, 1 = once per organism)
    pub max_task_count: [u32; 9],
}

impl TaskEnvironment {
    /// Create the default Logic-9 environment
    pub fn default_logic9() -> Self {
        Self {
            task_enabled: [true; 9], // All tasks enabled
            max_task_count: [1; 9],  // Each task can only be rewarded once
        }
    }

    /// Check if a task is enabled and can still be rewarded
    pub fn can_reward_task(&self, task: Task, current_count: u32) -> bool {
        let idx = task as usize;
        self.task_enabled[idx]
            && (self.max_task_count[idx] == 0 || current_count < self.max_task_count[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_task_merit_multipliers() {
        assert_eq!(Task::Not.merit_multiplier(), 2.0);
        assert_eq!(Task::Nand.merit_multiplier(), 2.0);
        assert_eq!(Task::And.merit_multiplier(), 4.0);
        assert_eq!(Task::Orn.merit_multiplier(), 4.0);
        assert_eq!(Task::Or.merit_multiplier(), 8.0);
        assert_eq!(Task::Andn.merit_multiplier(), 8.0);
        assert_eq!(Task::Nor.merit_multiplier(), 16.0);
        assert_eq!(Task::Xor.merit_multiplier(), 16.0);
        assert_eq!(Task::Equ.merit_multiplier(), 16.0);
    }

    #[test]
    fn test_not_task() {
        let mut detector = TaskDetector::new();
        detector.add_input(5);
        detector.add_input(10);

        let output = !10;
        let result = detector.check_output(output);
        assert!(result.is_some());
        let (task, inputs) = result.unwrap();
        assert_eq!(task, Task::Not);
        assert_eq!(inputs.len(), 2);
    }

    #[test]
    fn test_nand_task() {
        let mut detector = TaskDetector::new();
        detector.add_input(0b1100);
        detector.add_input(0b1010);

        let output = !(0b1100 & 0b1010);
        let result = detector.check_output(output);
        assert!(result.is_some());
        let (task, _) = result.unwrap();
        assert_eq!(task, Task::Nand);
    }

    #[test]
    fn test_and_task() {
        let mut detector = TaskDetector::new();
        detector.add_input(0b1100);
        detector.add_input(0b1010);

        let output = 0b1100 & 0b1010; // 0b1000
        let result = detector.check_output(output);
        assert!(result.is_some());
        let (task, _) = result.unwrap();
        assert_eq!(task, Task::And);
    }

    #[test]
    fn test_orn_task() {
        let mut detector = TaskDetector::new();
        detector.add_input(0b1100);
        detector.add_input(0b1010);

        let output = 0b1100 | !0b1010;
        let result = detector.check_output(output);
        assert!(result.is_some());
        let (task, _) = result.unwrap();
        assert_eq!(task, Task::Orn);
    }

    #[test]
    fn test_or_task() {
        let mut detector = TaskDetector::new();
        detector.add_input(0b1100);
        detector.add_input(0b1010);

        let output = 0b1100 | 0b1010;
        let result = detector.check_output(output);
        assert!(result.is_some());
        let (task, _) = result.unwrap();
        assert_eq!(task, Task::Or);
    }

    #[test]
    fn test_andn_task() {
        let mut detector = TaskDetector::new();
        detector.add_input(0b1100);
        detector.add_input(0b1010);

        let output = 0b1100 & !0b1010;
        let result = detector.check_output(output);
        assert!(result.is_some());
        let (task, _) = result.unwrap();
        assert_eq!(task, Task::Andn);
    }

    #[test]
    fn test_nor_task() {
        let mut detector = TaskDetector::new();
        detector.add_input(0b1100);
        detector.add_input(0b1010);

        let output = !(0b1100 | 0b1010);
        let result = detector.check_output(output);
        assert!(result.is_some());
        let (task, _) = result.unwrap();
        assert_eq!(task, Task::Nor);
    }

    #[test]
    fn test_xor_task() {
        let mut detector = TaskDetector::new();
        detector.add_input(0b1100);
        detector.add_input(0b1010);

        let output = 0b1100 ^ 0b1010;
        let result = detector.check_output(output);
        assert!(result.is_some());
        let (task, _) = result.unwrap();
        assert_eq!(task, Task::Xor);
    }

    #[test]
    fn test_equ_task() {
        let mut detector = TaskDetector::new();
        detector.add_input(0b1100);
        detector.add_input(0b1010);

        let output = !(0b1100 ^ 0b1010);
        let result = detector.check_output(output);
        assert!(result.is_some());
        let (task, _) = result.unwrap();
        assert_eq!(task, Task::Equ);
    }

    #[test]
    fn test_insufficient_inputs() {
        let detector = TaskDetector::new();
        let result = detector.check_output(42);
        assert!(result.is_none());

        let mut detector2 = TaskDetector::new();
        detector2.add_input(5);
        let result2 = detector2.check_output(42);
        assert!(result2.is_none());
    }

    #[test]
    fn test_task_detector_new() {
        let detector = TaskDetector::new();
        assert_eq!(detector.inputs.len(), 0);
    }

    #[test]
    fn test_task_detector_add_input() {
        let mut detector = TaskDetector::new();
        detector.add_input(10);
        detector.add_input(20);
        detector.add_input(30);
        assert_eq!(detector.inputs.len(), 3);
    }

    #[test]
    fn test_task_detector_input_buffer_limit() {
        let mut detector = TaskDetector::new();
        for i in 0..100 {
            detector.add_input(i);
        }
        // Should keep last inputs up to buffer size
        assert!(detector.inputs.len() <= 10);
    }

    #[test]
    fn test_wrong_output_no_task() {
        let mut detector = TaskDetector::new();
        detector.add_input(0b1100);
        detector.add_input(0b1010);

        // Output doesn't match any task
        let output = 42;
        let result = detector.check_output(output);
        assert!(result.is_none());
    }

    #[test]
    fn test_task_environment_default() {
        let env = TaskEnvironment::default_logic9();
        // All tasks should be enabled by default
        for i in 0..9 {
            assert!(env.task_enabled[i]);
        }
    }

    #[test]
    fn test_task_environment_task_limits() {
        let mut env = TaskEnvironment::default_logic9();
        env.max_task_count[0] = 5; // NOT task limit

        // Verify limit is set
        assert_eq!(env.max_task_count[0], 5);
    }

    #[test]
    fn test_task_environment_can_disable_task() {
        let mut env = TaskEnvironment::default_logic9();
        env.task_enabled[0] = false; // Disable NOT task

        assert!(!env.task_enabled[0]);
        assert!(env.task_enabled[1]); // Other tasks still enabled
    }

    #[test]
    fn test_multiple_tasks_same_inputs() {
        let mut detector = TaskDetector::new();
        let a = 0b1100;
        let b = 0b1010;
        detector.add_input(a);
        detector.add_input(b);

        // Test that different operations on same inputs detect different tasks
        let and_result = a & b;
        let or_result = a | b;
        let xor_result = a ^ b;

        let task1 = detector.check_output(and_result);
        assert_eq!(task1.unwrap().0, Task::And);

        let task2 = detector.check_output(or_result);
        assert_eq!(task2.unwrap().0, Task::Or);

        let task3 = detector.check_output(xor_result);
        assert_eq!(task3.unwrap().0, Task::Xor);
    }

    #[test]
    fn test_task_with_zero_inputs() {
        let mut detector = TaskDetector::new();
        detector.add_input(0);
        detector.add_input(0);

        let output = 0 & 0;
        let result = detector.check_output(output);
        assert!(result.is_some());
        let (task, _) = result.unwrap();
        assert_eq!(task, Task::And);
    }

    #[test]
    fn test_task_with_negative_inputs() {
        let mut detector = TaskDetector::new();
        detector.add_input(-5);
        detector.add_input(10);

        let output = !10;
        let result = detector.check_output(output);
        assert!(result.is_some());
    }
}
