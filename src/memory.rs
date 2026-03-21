use anyhow::{Result, anyhow};
use std::fs;

pub struct MemoryAccess {
    pub instruction_pointer: usize,
    pub code_segment: usize,
}

impl Clone for MemoryAccess {
    fn clone(&self) -> Self {
        return Self {
            instruction_pointer: self.instruction_pointer,
            code_segment: self.code_segment,
        };
    }
}

impl Copy for MemoryAccess {}

impl MemoryAccess {
    pub fn new() -> MemoryAccess {
        MemoryAccess {
            instruction_pointer: 0,
            code_segment: 0,
        }
    }

    pub fn absolute_address(&self) -> usize {
        self.instruction_pointer
    }
}

#[derive(Debug)]
pub struct Memory {
    data: Vec<u8>,
    program_size: usize,
}

const MAX_MEMORY: usize = 1024 * 1024;

impl Memory {
    pub fn load_program_binary(path: &str) -> Result<Memory> {
        let readed_binary = fs::read(path)?;
        let mut memory_vec = vec![0; MAX_MEMORY];
        for n in &readed_binary {
            print!("{:b} ", n);
        }
        println!();

        if readed_binary.len() >= MAX_MEMORY {
            return Err(anyhow!(
                "the loaded binary must be smaller than 1048576 bytes ~ 1 MiB"
            ));
        }

        // Copies all elements from src (readed_binary) into self (memory_vec), using a memcpy.
        // The length of src must be the same
        memory_vec[0..readed_binary.len()].copy_from_slice(&readed_binary[..]);

        Ok(Memory {
            data: memory_vec,
            program_size: readed_binary.len(),
        })
    }

    pub fn program_size(&self) -> usize {
        return self.program_size;
    }

    pub fn read(&self, memory_access: MemoryAccess) -> Result<&u8> {
        // TODO: Calculate effective address using instruction pointer and code segment.
        //
        // The absolute address position at wich CPU is reading instructions from.
        // Note the IP register is 16 bits and a 8086 program can access 1mb of data.
        // With 16 bits we only are able to address 2^16 = 65536 different address locations.
        // So we they solved the problem combining code segment and IP registers. Convention is (CS:IP)
        // To produce a 20 bits numbers. Formula is the following:
        // AbsolutePosition = (cs << 4) | ip.
        let res = self.data.get(memory_access.instruction_pointer);
        // TODO: How should we simulate out of range access, maybe wrap?
        match res {
            Some(val) => return Ok(val),
            None => return Err(anyhow!("accessing invalid memory")),
        }
    }
}
