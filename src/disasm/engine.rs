//! Disassembly Engine - iced-x86 wrapper
//!
//! Provides high-level disassembly interface with formatting options.

use iced_x86::{
    Decoder, DecoderOptions, Formatter, GasFormatter, Instruction, IntelFormatter, MasmFormatter,
    NasmFormatter,
};
use thiserror::Error;

/// Disassembly errors
#[derive(Error, Debug)]
pub enum DisasmError {
    #[error("Invalid instruction at offset {offset:#x}")]
    InvalidInstruction { offset: u64 },

    #[error("Buffer too small")]
    BufferTooSmall,

    #[error("Unsupported architecture: {0}")]
    UnsupportedArch(String),
}

/// Disassembly output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyntaxFormat {
    #[default]
    Intel, // mov eax, [ebx+4]
    Masm, // mov eax, DWORD PTR [ebx+4]
    Nasm, // mov eax, dword [ebx+4]
    Gas,  // movl 4(%ebx), %eax
}

/// Target architecture bitness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Bitness {
    Bit16 = 16,
    Bit32 = 32,
    #[default]
    Bit64 = 64,
}

/// A single disassembled instruction
#[derive(Debug, Clone)]
pub struct DisassembledInstruction {
    /// Address of the instruction
    pub address: u64,

    /// Raw bytes of the instruction
    pub bytes: Vec<u8>,

    /// Formatted instruction string
    pub mnemonic: String,

    /// Length of the instruction in bytes
    pub length: usize,

    /// Whether this is a branch instruction
    pub is_branch: bool,

    /// Whether this is a call instruction
    pub is_call: bool,

    /// Whether this is a return instruction
    pub is_ret: bool,

    /// Branch/call target address (if applicable)
    pub target_address: Option<u64>,
}

/// Main disassembly engine
pub struct DisassemblyEngine {
    /// Target architecture bitness
    bitness: Bitness,

    /// Output syntax format
    syntax: SyntaxFormat,
}

impl DisassemblyEngine {
    /// Create a new disassembly engine with default settings (64-bit, Intel syntax)
    pub fn new() -> Self {
        Self {
            bitness: Bitness::Bit64,
            syntax: SyntaxFormat::Intel,
        }
    }

    /// Create a new engine with specified bitness
    pub fn with_bitness(bitness: Bitness) -> Self {
        Self {
            bitness,
            syntax: SyntaxFormat::Intel,
        }
    }

    /// Set the output syntax format
    pub fn set_syntax(&mut self, syntax: SyntaxFormat) {
        self.syntax = syntax;
    }

    /// Set the target architecture bitness
    pub fn set_bitness(&mut self, bitness: Bitness) {
        self.bitness = bitness;
    }

    /// Disassemble a buffer of bytes
    pub fn disassemble(&self, bytes: &[u8], base_address: u64) -> Vec<DisassembledInstruction> {
        let mut decoder = Decoder::with_ip(
            self.bitness as u32,
            bytes,
            base_address,
            DecoderOptions::NONE,
        );

        let mut instructions = Vec::new();
        let mut formatter = self.create_formatter();

        while decoder.can_decode() {
            let instruction: Instruction = decoder.decode();

            if instruction.is_invalid() {
                // Skip invalid instructions but track them
                instructions.push(DisassembledInstruction {
                    address: instruction.ip(),
                    bytes: vec![bytes[(instruction.ip() - base_address) as usize]],
                    mnemonic: "db 0x??".to_string(),
                    length: 1,
                    is_branch: false,
                    is_call: false,
                    is_ret: false,
                    target_address: None,
                });
                continue;
            }

            let start = (instruction.ip() - base_address) as usize;
            let end = start + instruction.len();
            let raw_bytes = bytes[start..end].to_vec();

            // Format the instruction
            let mut output = String::new();
            formatter.format(&instruction, &mut output);

            // Analyze instruction type
            let is_branch = instruction.is_jcc_short_or_near()
                || instruction.is_jmp_short_or_near()
                || instruction.is_jmp_far();
            let is_call = instruction.is_call_near() || instruction.is_call_far();
            let is_ret = instruction.mnemonic() == iced_x86::Mnemonic::Ret
                || instruction.mnemonic() == iced_x86::Mnemonic::Retf;

            // Get branch/call target if applicable
            let target_address = if is_branch || is_call {
                if instruction.is_ip_rel_memory_operand() {
                    Some(instruction.ip_rel_memory_address())
                } else if instruction.near_branch_target() != 0 {
                    Some(instruction.near_branch_target())
                } else {
                    None
                }
            } else {
                None
            };

            instructions.push(DisassembledInstruction {
                address: instruction.ip(),
                bytes: raw_bytes,
                mnemonic: output,
                length: instruction.len(),
                is_branch,
                is_call,
                is_ret,
                target_address,
            });
        }

        instructions
    }

    /// Disassemble a single instruction at the given address
    pub fn disassemble_one(&self, bytes: &[u8], address: u64) -> Option<DisassembledInstruction> {
        let instructions = self.disassemble(bytes, address);
        instructions.into_iter().next()
    }

    /// Format instructions as a string table (for display)
    pub fn format_listing(&self, instructions: &[DisassembledInstruction]) -> String {
        let mut output = String::new();

        for inst in instructions {
            // Format: ADDRESS  BYTES                 INSTRUCTION
            let bytes_str: String = inst
                .bytes
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            output.push_str(&format!(
                "{:016X}  {:24}  {}\n",
                inst.address, bytes_str, inst.mnemonic
            ));
        }

        output
    }

    /// Create a formatter based on the current syntax setting
    fn create_formatter(&self) -> Box<dyn Formatter> {
        match self.syntax {
            SyntaxFormat::Intel => Box::new(IntelFormatter::new()),
            SyntaxFormat::Masm => Box::new(MasmFormatter::new()),
            SyntaxFormat::Nasm => Box::new(NasmFormatter::new()),
            SyntaxFormat::Gas => Box::new(GasFormatter::new()),
        }
    }
}

impl Default for DisassemblyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble_basic() {
        let engine = DisassemblyEngine::new();

        // push rbp; mov rbp, rsp; sub rsp, 0x20
        let bytes = [0x55, 0x48, 0x89, 0xE5, 0x48, 0x83, 0xEC, 0x20];
        let instructions = engine.disassemble(&bytes, 0x401000);

        assert_eq!(instructions.len(), 3);
        assert_eq!(instructions[0].mnemonic, "push rbp");
        assert_eq!(instructions[1].mnemonic, "mov rbp,rsp");
    }
}
