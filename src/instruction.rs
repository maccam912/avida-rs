/// Avida instruction set - 26 instructions (a-z)
/// Based on the default instruction set from original Avida
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Instruction {
    // No-operation instructions (a-c)
    NopA,  // a - no-op, modifies previous instruction or acts as label
    NopB,  // b - no-op, modifies previous instruction or acts as label
    NopC,  // c - no-op, modifies previous instruction or acts as label

    // Conditional operations (d-e)
    IfNEqu,    // d - if BX != complement label, execute next
    IfLess,    // e - if BX < complement, execute next

    // Stack operations (f-h)
    Pop,       // f - pop stack top into BX
    Push,      // g - push BX onto active stack
    SwapStk,   // h - toggle between two stacks

    // Register operations (i-m)
    Swap,      // i - exchange BX with complement register
    ShiftR,    // j - right bit-shift BX (divide by 2)
    ShiftL,    // k - left bit-shift BX (multiply by 2)
    Inc,       // l - increment BX
    Dec,       // m - decrement BX

    // Arithmetic (n-p)
    Add,       // n - BX = BX + CX
    Sub,       // o - BX = BX - CX
    Nand,      // p - BX = BX NAND CX (bitwise)

    // I/O (q)
    IO,        // q - output BX, check tasks, input new value

    // Genome management (r-u)
    HAlloc,    // r - allocate memory for offspring
    HDivide,   // s - divide organism, create offspring
    HCopy,     // t - copy instruction from read-head to write-head
    HSearch,   // u - find complement label template

    // Head movement (v-x)
    MovHead,   // v - jump IP to flow-head position
    JmpHead,   // w - move IP by CX register amount
    GetHead,   // x - copy IP position to CX register

    // Flow control (y-z)
    IfLabel,   // y - test if complement label was most recently copied
    SetFlow,   // z - move flow-head to position in CX register
}

impl Instruction {
    /// Convert a character to an instruction
    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'a' => Some(Instruction::NopA),
            'b' => Some(Instruction::NopB),
            'c' => Some(Instruction::NopC),
            'd' => Some(Instruction::IfNEqu),
            'e' => Some(Instruction::IfLess),
            'f' => Some(Instruction::Pop),
            'g' => Some(Instruction::Push),
            'h' => Some(Instruction::SwapStk),
            'i' => Some(Instruction::Swap),
            'j' => Some(Instruction::ShiftR),
            'k' => Some(Instruction::ShiftL),
            'l' => Some(Instruction::Inc),
            'm' => Some(Instruction::Dec),
            'n' => Some(Instruction::Add),
            'o' => Some(Instruction::Sub),
            'p' => Some(Instruction::Nand),
            'q' => Some(Instruction::IO),
            'r' => Some(Instruction::HAlloc),
            's' => Some(Instruction::HDivide),
            't' => Some(Instruction::HCopy),
            'u' => Some(Instruction::HSearch),
            'v' => Some(Instruction::MovHead),
            'w' => Some(Instruction::JmpHead),
            'x' => Some(Instruction::GetHead),
            'y' => Some(Instruction::IfLabel),
            'z' => Some(Instruction::SetFlow),
            _ => None,
        }
    }

    /// Convert an instruction to its character representation
    pub fn to_char(self) -> char {
        match self {
            Instruction::NopA => 'a',
            Instruction::NopB => 'b',
            Instruction::NopC => 'c',
            Instruction::IfNEqu => 'd',
            Instruction::IfLess => 'e',
            Instruction::Pop => 'f',
            Instruction::Push => 'g',
            Instruction::SwapStk => 'h',
            Instruction::Swap => 'i',
            Instruction::ShiftR => 'j',
            Instruction::ShiftL => 'k',
            Instruction::Inc => 'l',
            Instruction::Dec => 'm',
            Instruction::Add => 'n',
            Instruction::Sub => 'o',
            Instruction::Nand => 'p',
            Instruction::IO => 'q',
            Instruction::HAlloc => 'r',
            Instruction::HDivide => 's',
            Instruction::HCopy => 't',
            Instruction::HSearch => 'u',
            Instruction::MovHead => 'v',
            Instruction::JmpHead => 'w',
            Instruction::GetHead => 'x',
            Instruction::IfLabel => 'y',
            Instruction::SetFlow => 'z',
        }
    }

    /// Get the complement nop instruction for template matching
    /// nop-A complements nop-B
    /// nop-B complements nop-C
    /// nop-C complements nop-A
    pub fn complement_nop(self) -> Option<Self> {
        match self {
            Instruction::NopA => Some(Instruction::NopB),
            Instruction::NopB => Some(Instruction::NopC),
            Instruction::NopC => Some(Instruction::NopA),
            _ => None,
        }
    }

    /// Check if this is a nop instruction
    pub fn is_nop(self) -> bool {
        matches!(self, Instruction::NopA | Instruction::NopB | Instruction::NopC)
    }

    /// Get the register index for this nop (used for register selection)
    /// nop-A = 0 (AX), nop-B = 1 (BX), nop-C = 2 (CX)
    pub fn nop_register_index(self) -> Option<usize> {
        match self {
            Instruction::NopA => Some(0),
            Instruction::NopB => Some(1),
            Instruction::NopC => Some(2),
            _ => None,
        }
    }
}

/// Parse a genome string into a vector of instructions
pub fn parse_genome(s: &str) -> Result<Vec<Instruction>, String> {
    s.chars()
        .map(|c| Instruction::from_char(c)
            .ok_or_else(|| format!("Invalid instruction character: '{}'", c)))
        .collect()
}

/// Convert a genome vector back to a string
pub fn genome_to_string(genome: &[Instruction]) -> String {
    genome.iter().map(|i| i.to_char()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_char_all_instructions() {
        assert_eq!(Instruction::from_char('a'), Some(Instruction::NopA));
        assert_eq!(Instruction::from_char('b'), Some(Instruction::NopB));
        assert_eq!(Instruction::from_char('c'), Some(Instruction::NopC));
        assert_eq!(Instruction::from_char('d'), Some(Instruction::IfNEqu));
        assert_eq!(Instruction::from_char('e'), Some(Instruction::IfLess));
        assert_eq!(Instruction::from_char('f'), Some(Instruction::Pop));
        assert_eq!(Instruction::from_char('g'), Some(Instruction::Push));
        assert_eq!(Instruction::from_char('h'), Some(Instruction::SwapStk));
        assert_eq!(Instruction::from_char('i'), Some(Instruction::Swap));
        assert_eq!(Instruction::from_char('j'), Some(Instruction::ShiftR));
        assert_eq!(Instruction::from_char('k'), Some(Instruction::ShiftL));
        assert_eq!(Instruction::from_char('l'), Some(Instruction::Inc));
        assert_eq!(Instruction::from_char('m'), Some(Instruction::Dec));
        assert_eq!(Instruction::from_char('n'), Some(Instruction::Add));
        assert_eq!(Instruction::from_char('o'), Some(Instruction::Sub));
        assert_eq!(Instruction::from_char('p'), Some(Instruction::Nand));
        assert_eq!(Instruction::from_char('q'), Some(Instruction::IO));
        assert_eq!(Instruction::from_char('r'), Some(Instruction::HAlloc));
        assert_eq!(Instruction::from_char('s'), Some(Instruction::HDivide));
        assert_eq!(Instruction::from_char('t'), Some(Instruction::HCopy));
        assert_eq!(Instruction::from_char('u'), Some(Instruction::HSearch));
        assert_eq!(Instruction::from_char('v'), Some(Instruction::MovHead));
        assert_eq!(Instruction::from_char('w'), Some(Instruction::JmpHead));
        assert_eq!(Instruction::from_char('x'), Some(Instruction::GetHead));
        assert_eq!(Instruction::from_char('y'), Some(Instruction::IfLabel));
        assert_eq!(Instruction::from_char('z'), Some(Instruction::SetFlow));
    }

    #[test]
    fn test_from_char_uppercase() {
        assert_eq!(Instruction::from_char('A'), Some(Instruction::NopA));
        assert_eq!(Instruction::from_char('R'), Some(Instruction::HAlloc));
        assert_eq!(Instruction::from_char('Z'), Some(Instruction::SetFlow));
    }

    #[test]
    fn test_from_char_invalid() {
        assert_eq!(Instruction::from_char('1'), None);
        assert_eq!(Instruction::from_char('!'), None);
        assert_eq!(Instruction::from_char(' '), None);
        assert_eq!(Instruction::from_char('0'), None);
    }

    #[test]
    fn test_to_char_all_instructions() {
        assert_eq!(Instruction::NopA.to_char(), 'a');
        assert_eq!(Instruction::NopB.to_char(), 'b');
        assert_eq!(Instruction::NopC.to_char(), 'c');
        assert_eq!(Instruction::HAlloc.to_char(), 'r');
        assert_eq!(Instruction::HDivide.to_char(), 's');
        assert_eq!(Instruction::HCopy.to_char(), 't');
        assert_eq!(Instruction::SetFlow.to_char(), 'z');
    }

    #[test]
    fn test_roundtrip_conversion() {
        for c in 'a'..='z' {
            if let Some(inst) = Instruction::from_char(c) {
                assert_eq!(inst.to_char(), c);
            }
        }
    }

    #[test]
    fn test_complement_nop_cycle() {
        assert_eq!(Instruction::NopA.complement_nop(), Some(Instruction::NopB));
        assert_eq!(Instruction::NopB.complement_nop(), Some(Instruction::NopC));
        assert_eq!(Instruction::NopC.complement_nop(), Some(Instruction::NopA));
    }

    #[test]
    fn test_complement_nop_non_nops() {
        assert_eq!(Instruction::Add.complement_nop(), None);
        assert_eq!(Instruction::HAlloc.complement_nop(), None);
        assert_eq!(Instruction::IO.complement_nop(), None);
    }

    #[test]
    fn test_is_nop() {
        assert!(Instruction::NopA.is_nop());
        assert!(Instruction::NopB.is_nop());
        assert!(Instruction::NopC.is_nop());
        assert!(!Instruction::Add.is_nop());
        assert!(!Instruction::HAlloc.is_nop());
        assert!(!Instruction::IO.is_nop());
    }

    #[test]
    fn test_nop_register_index() {
        assert_eq!(Instruction::NopA.nop_register_index(), Some(0));
        assert_eq!(Instruction::NopB.nop_register_index(), Some(1));
        assert_eq!(Instruction::NopC.nop_register_index(), Some(2));
        assert_eq!(Instruction::Add.nop_register_index(), None);
        assert_eq!(Instruction::HAlloc.nop_register_index(), None);
    }

    #[test]
    fn test_parse_genome_valid() {
        let result = parse_genome("abc");
        assert!(result.is_ok());
        let genome = result.unwrap();
        assert_eq!(genome.len(), 3);
        assert_eq!(genome[0], Instruction::NopA);
        assert_eq!(genome[1], Instruction::NopB);
        assert_eq!(genome[2], Instruction::NopC);
    }

    #[test]
    fn test_parse_genome_empty() {
        let result = parse_genome("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_parse_genome_invalid() {
        let result = parse_genome("abc123");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid instruction character"));
    }

    #[test]
    fn test_parse_genome_ancestor() {
        let ancestor = "rutyabsvacccccccccccccccccccccccccccccccccccccccbc";
        let result = parse_genome(ancestor);
        assert!(result.is_ok());
        let genome = result.unwrap();
        assert_eq!(genome.len(), 50);
        assert_eq!(genome[0], Instruction::HAlloc);      // r
        assert_eq!(genome[1], Instruction::HSearch);     // u
        assert_eq!(genome[2], Instruction::HCopy);       // t
        assert_eq!(genome[3], Instruction::IfLabel);     // y
    }

    #[test]
    fn test_genome_to_string() {
        let genome = vec![
            Instruction::NopA,
            Instruction::NopB,
            Instruction::NopC,
            Instruction::HAlloc,
        ];
        assert_eq!(genome_to_string(&genome), "abcr");
    }

    #[test]
    fn test_genome_to_string_empty() {
        let genome: Vec<Instruction> = vec![];
        assert_eq!(genome_to_string(&genome), "");
    }

    #[test]
    fn test_genome_roundtrip() {
        let original = "rutyabsvacccccccccccccccccccccccccccccccccccccccbc";
        let genome = parse_genome(original).unwrap();
        let result = genome_to_string(&genome);
        assert_eq!(result, original);
    }

    #[test]
    fn test_genome_roundtrip_all_instructions() {
        let all = "abcdefghijklmnopqrstuvwxyz";
        let genome = parse_genome(all).unwrap();
        let result = genome_to_string(&genome);
        assert_eq!(result, all);
    }

    #[test]
    fn test_instruction_copy_clone() {
        let inst = Instruction::NopA;
        let copied = inst;
        assert_eq!(inst, copied);

        let cloned = inst.clone();
        assert_eq!(inst, cloned);
    }

    #[test]
    fn test_instruction_debug() {
        let inst = Instruction::NopA;
        let debug_str = format!("{:?}", inst);
        assert!(debug_str.contains("NopA"));
    }
}
