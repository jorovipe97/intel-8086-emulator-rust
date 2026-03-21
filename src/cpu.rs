pub struct Cpu {
    instruction_pointer: i32,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            instruction_pointer: 0,
        }
    }

    pub fn instruction_pointer(&self) -> i32 {
        return self.instruction_pointer;
    }
}
