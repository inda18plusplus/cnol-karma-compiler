
use llvm_sys::prelude::*;
use builder::*;
use karma_parser::*;


/// Builds a sequence
pub struct SequenceBuilder<'a> {
    builder: &'a mut Builder,
    target_fn: LLVMValueRef,
    panic_block: LLVMBasicBlockRef,
    success_block: LLVMBasicBlockRef
}

pub struct SequenceBlock {
    // the jump table into the sequence
    pub jump_table: LLVMBasicBlockRef,

    // i64*, the next block to execute in the sequence
    pub next_section: LLVMValueRef
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
        unsafe { self.create_sequence_blocks(sequences) } 
    }
}


// Implementation
impl<'a> SequenceBuilder<'a> {
    unsafe fn create_sequence_blocks(&mut self, sequences: &[Sequence]) -> Vec<SequenceBlock> {
        let mut sequence_blocks = Vec::new();

        for (i, sequence) in sequences.iter().enumerate() {
            let jump_table = self.builder.add_block(self.target_fn, &format!("jump_table_{}", i));
            let (next_section, sections) = self.create_section_blocks(jump_table, i, sequence);

            self.build_jump_table(jump_table, sections, next_section);

            sequence_blocks.push(SequenceBlock {
                jump_table, next_section
            });
        }

        sequence_blocks
    }


    /// Creates and builds a block for each section and a global variable
    /// specifying which section to execute after the next jump.
    unsafe fn create_section_blocks(&mut self,
                             jump_table: LLVMBasicBlockRef,
                             sequence_number: usize,
                             sequence: &Sequence) -> (LLVMValueRef, Vec<LLVMBasicBlockRef>) {
        // Keeps track of the next section to jump to in the jump table
        let next_section = self.builder.add_global_variable(&format!("next_section_{}", sequence_number), i64_value(0));

        let sections = sequence.iter().enumerate().map(|(i, section)| {
            let name = &format!("section_{}_{}", sequence_number, i);
            let block = self.builder.add_block(self.target_fn, name);

            self.builder.build_block(block, |mut b| {
                b.store(i64_value(i as i64 + 1), next_section);
            });

            let mut exit = false;

            for instruction in section.iter() {
                self.add_instruction(block, instruction);

                match instruction {
                    &Instruction::Exit => exit = true,
                    _ => ()
                };
            }

            if !exit {
                // TODO: Branch to the correct section directly
                self.builder.build_block(block, |mut b| {
                    b.branch(jump_table);
                });
            }

            block
        }).collect();

        (next_section, sections)
    }



    unsafe fn build_jump_table(&mut self,
                        jump_table: LLVMBasicBlockRef,
                        sections: Vec<LLVMBasicBlockRef>,
                        next_section: LLVMValueRef) {
        let exit = self.panic_block;
        self.builder.build_block(jump_table, |mut b| {
            let next = b.load(next_section, "");
            let numbered_sections: Vec<_> = sections.into_iter().enumerate()
                .map(|(i, section)| (i64_value(i as i64), section))
                .collect();

            b.switch(next, &numbered_sections, exit);
        });
    }


    unsafe fn add_instruction(&mut self,
                       block: LLVMBasicBlockRef,
                       instruction: &Instruction) {
        match instruction {
            &Instruction::Push(ref source) => self.add_push(block, source),
            &Instruction::OutputCharacter(ref source) => self.add_output_character(block, source),
            &Instruction::OutputNumber(ref source) => self.add_output_number(block, source),

            &Instruction::Exit => self.add_exit(block),

            _ => unimplemented!("Instruction {:?}", instruction)
        }
    }




    unsafe fn add_push(&mut self,
                block: LLVMBasicBlockRef,
                source: &ValueSource) {
        self.builder.build_block(block, |mut b| {
            let value = Self::get_value_from_source(&mut b, source);
            b.call_function("push", &[value], "");
        })
    }
    
    unsafe fn add_output_character(&mut self,
                            block: LLVMBasicBlockRef,
                            source: &ValueSource) {
        self.builder.build_block(block, |mut b| {
            let value = Self::get_value_from_source(&mut b, source);
            let value = b.cast_int(value, i32_type(), "");
            b.call_function("putchar", &[value], "");
        })
    }

    unsafe fn add_output_number(&mut self,
                         block: LLVMBasicBlockRef,
                         source: &ValueSource) {
        self.builder.build_block(block, |mut b| {
            let value = Self::get_value_from_source(&mut b, source);
            b.call_function("puti64", &[value], "");
        })
    }

    unsafe fn add_exit(&mut self,
                block: LLVMBasicBlockRef) {
        let exit = self.success_block;
        self.builder.build_block(block, |mut b| {
            b.branch(exit);
        })
    }






    unsafe fn get_value_from_source(builder: &mut BlockBuilder,
                             source: &ValueSource) -> LLVMValueRef {
        match source {
            &ValueSource::Digit(digit) => i64_value(digit as i64),
            &ValueSource::Pop => builder.call_function("pop", &[], ""),

            &ValueSource::Operate(ref lhs, ref operation, ref rhs) => {
                let lhs_value = Self::get_value_from_source(builder, lhs);
                let rhs_value = Self::get_value_from_source(builder, rhs);
                Self::add_operation(builder, lhs_value, operation, rhs_value)
            },

            _ => unimplemented!("Value Source: {:?}", source)
        }
    }

    unsafe fn add_operation(builder: &mut BlockBuilder,
                     lhs: LLVMValueRef,
                     op: &Operator,
                     rhs: LLVMValueRef) -> LLVMValueRef {
        match op {
            _ => unimplemented!("Operator: {:?}", op)
        }
    }
}

/*


   fn add_instruction(builder: &mut Builder,
   section: LLVMBasicBlockRef,
   instruction: &Instruction) {
   match instruction {
   &Instruction::Push(ref source) => add_push(builder, section, source),
   &Instruction::OutputCharacter(ref source) => add_output_character(builder, section, source),
   &Instruction::OutputNumber(ref source) => add_output_number(builder, section, source),

   &Instruction::Exit => add_exit(builder, section),

   _ => unimplemented!("Instruction {:?}", instruction)
   }
   }


   fn add_push(builder: &mut Builder,
   section: LLVMBasicBlockRef,
   source: &ValueSource) {
   builder.build_block(section, |mut b| {
   let value = get_value_from_source(&mut b, source);
   b.call_function("push", &[value], "");
   })
   }

   fn add_output_character(builder: &mut Builder,
   section: LLVMBasicBlockRef,
   source: &ValueSource) {
   builder.build_block(section, |mut b| {
   let value = get_value_from_source(&mut b, source);
   let value = b.cast_int(value, i32_type(), "");
   b.call_function("putchar", &[value], "");
   })
   }

   fn add_output_number(builder: &mut Builder,
   section: LLVMBasicBlockRef,
   source: &ValueSource) {
   builder.build_block(section, |mut b| {
   let value = get_value_from_source(&mut b, source);
   b.call_function("puti64", &[value], "");
   })
   }

   fn add_exit(builder: &mut Builder,
   section: LLVMBasicBlockRef) {
   builder.build_block(section, |mut b| {
   b.return_value(i32_value(0));
   })
   }


   fn get_value_from_source(builder: &mut BlockBuilder,
   section: LLVMBasicBlockRef,
   source: &ValueSource) -> LLVMValueRef {
   match source {
   &ValueSource::Digit(digit) => i64_value(digit as i64),
   &ValueSource::Pop => builder.call_function("pop", &[], ""),

   &ValueSource::Operate(ref lhs, ref operation, ref rhs) => {
   let lhs_value = get_value_from_source(builder, lhs);
   let rhs_value = get_value_from_source(builder, rhs);
   add_operation(builder, lhs_value, operation, rhs_value)
   },

   _ => unimplemented!("Value Source: {:?}", source)
   }
   }




*/
