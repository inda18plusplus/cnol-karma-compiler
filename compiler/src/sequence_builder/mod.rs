
use llvm_sys::prelude::*;
use builder::*;
use karma_parser::*;

mod instruction_builder;
use self::instruction_builder::*;


/// Builds a sequence
pub struct SequenceBuilder<'a> {
    builder: &'a mut Builder,
    target_fn: LLVMValueRef,
    panic_block: LLVMBasicBlockRef,
    success_block: LLVMBasicBlockRef
}

pub struct SequenceBlock {
    // the jump table into the sequence
    pub jump_table: JumpTable,

    // All sections
    sections: Vec<LLVMBasicBlockRef>
}

pub struct JumpTable {
    pub block: LLVMBasicBlockRef,
    pub next_section: LLVMValueRef,
}


// Interface
impl<'a> SequenceBuilder<'a> {
    pub fn new(builder: &'a mut Builder,
               target_fn: LLVMValueRef,
               panic_block: LLVMBasicBlockRef,
               success_block: LLVMBasicBlockRef) -> Self {
        SequenceBuilder {
            builder,
            target_fn,
            panic_block,
            success_block
        }
    }


    pub fn build(mut self, sequences: &[Sequence]) -> Vec<SequenceBlock> {
        let sequence_blocks = self.create_sequence_blocks(sequences);

        self.build_sequence_blocks(&sequence_blocks, sequences);

        sequence_blocks
    }
}


// Implementation
impl<'a> SequenceBuilder<'a> {
    /// Creates all blocks for each sequence
    fn create_sequence_blocks(&mut self, sequences: &[Sequence]) -> Vec<SequenceBlock> {
        let mut sequence_blocks = Vec::new();

        for (i, sequence) in sequences.iter().enumerate() {
            let sections = self.create_sections(i, sequence);
            let jump_table = self.build_jump_table(i, &sections);

            sequence_blocks.push(SequenceBlock {
                jump_table, sections
            });
        }

        sequence_blocks
    }
    

    /// Creates a block for each section
    fn create_sections(&mut self,
                       sequence_number: usize,
                       sequence: &Sequence) -> Vec<LLVMBasicBlockRef> {
        sequence.iter().enumerate().map(|(i, _)| {
            let name = &format!("section_{}_{}", sequence_number, i);
            let block = self.builder.add_block(self.target_fn, name);

            block
        }).collect()
    }


    fn build_jump_table(&mut self,
                        sequence_number: usize,
                        sections: &[LLVMBasicBlockRef]) -> JumpTable {
        let table_name = format!("jump_table_{}", sequence_number);
        let next_name = format!("next_section_{}", sequence_number);

        let jump_table = self.builder.add_block(self.target_fn, &table_name);
        let next_section = self.builder.add_global_variable(&next_name, i64_value(0));

        let exit = self.panic_block;

        self.builder.build_block(jump_table, |mut b| {
            let next = b.load(next_section);
            let numbered_sections: Vec<_> = sections.iter().enumerate()
                .map(|(i, section)| (i64_value(i as i64), *section))
                .collect();

            b.switch(next, &numbered_sections, exit);
        });

        JumpTable {
            block: jump_table,
            next_section
        }
    }


    fn build_sequence_blocks(&mut self, sequence_blocks: &[SequenceBlock], sequences: &[Sequence]) {
        let on_success = self.success_block;
        let on_failure = self.panic_block;

        for (sequence_index, (sequence_block, sequence)) in sequence_blocks.iter().zip(sequences.iter()).enumerate() {
            for (section_index, (block, section)) in sequence_block.sections.iter().zip(sequence.iter()).enumerate() {
                self.builder.build_block(*block, |builder| {
                    let instructions = section.as_slice();

                    InstructionBuilder {
                        builder,
                        on_success,
                        on_failure,
                        sequences: sequence_blocks,
                        sequence: sequence_index,
                        section: section_index
                    }.build(instructions);
                });
            }
        }
    }
}

