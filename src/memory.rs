use anyhow::{Context, Result, anyhow};
use std::fs;

pub struct MemoryAccess {
    pub offset: usize,
    pub segment: usize,
}

impl Clone for MemoryAccess {
    fn clone(&self) -> Self {
        return Self {
            offset: self.offset,
            segment: self.segment,
        };
    }
}

impl Copy for MemoryAccess {}

impl MemoryAccess {
    pub fn new() -> MemoryAccess {
        MemoryAccess {
            offset: 0,
            segment: 0,
        }
    }

    pub fn absolute_address(&self) -> usize {
        self.offset
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

        // Uncomment to debug binary of loaded program.
        for n in &readed_binary {
            print!("{:08b} ", n);
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

    pub fn load_byte(&self, memory_access: MemoryAccess) -> Result<u8> {
        // TODO: Calculate effective address using instruction pointer and code segment.
        //
        // The absolute address position at wich CPU is reading instructions from.
        // Note the IP register is 16 bits and a 8086 program can access 1mb of data.
        // With 16 bits we only are able to address 2^16 = 65536 different address locations.
        // So we they solved the problem combining code segment and IP registers. Convention is (CS:IP)
        // To produce a 20 bits numbers. Formula is the following:
        // AbsolutePosition = (cs << 4) + ip.
        let absolute_position = (memory_access.segment << 4).wrapping_add(memory_access.offset);
        let res = *self
            .data
            .get(absolute_position)
            .ok_or_else(|| anyhow!("accessing invalid memory"))?;

        Ok(res)
    }

    pub fn store_byte(&mut self, memory_access: MemoryAccess, value: u8) -> Result<()> {
        let absolute_position = (memory_access.segment << 4).wrapping_add(memory_access.offset);

        let mem_address = self
            .data
            .get_mut(absolute_position)
            .ok_or_else(|| anyhow!("accessing invalid memory"))?;
        *mem_address = value;

        Ok(())
    }

    pub fn load_word(&self, memory_access: MemoryAccess) -> Result<u16> {
        let absolute_position = (memory_access.segment << 4).wrapping_add(memory_access.offset);
        // vec.get() returns a refrence (borrow) to the read item, we copy it
        // as a store may write the same location, and we dont want api users to fight the borrow
        // system there as everything is possible with the simulated memory.
        let low_byte = *self
            .data
            .get(absolute_position)
            .ok_or_else(|| anyhow!("accessing invalid memory"))? as u16;
        let high_byte = *self
            .data
            .get(absolute_position.wrapping_add(1))
            .ok_or_else(|| anyhow!("accessing invalid memory"))? as u16;

        let result = high_byte << 8 | low_byte;
        return Ok(result);
    }

    pub fn store_word(&mut self, memory_access: MemoryAccess, value: u16) -> Result<()> {
        let absolute_position = (memory_access.segment << 4).wrapping_add(memory_access.offset);

        // Intel 8086 is little endiang, meaning the lowest significative bytes are stored in lower memory addresses.
        let high_byte = (value >> 8) as u8;
        let low_byte = value as u8;

        let low_mem_address = self
            .data
            .get_mut(absolute_position)
            .ok_or_else(|| anyhow!("accessing invalid memory in low memory address of word"))?;
        *low_mem_address = low_byte;

        let high_mem_address = self
            .data
            .get_mut(absolute_position.wrapping_add(1))
            .ok_or_else(|| anyhow!("accessing invalid memory in high memory address of word"))?;
        *high_mem_address = high_byte;

        Ok(())
    }

    // Saves entire memory into a file.
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        fs::write(path, &self.data).with_context(|| "failed to save memory content into a file")?;

        Ok(())
    }
}
