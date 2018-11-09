

use llvm_sys::prelude::*;
use builder::*;
use karma_parser::*;

use super::SequenceBlock;

pub struct InstructionBuilder<'a> {
    pub builder: BlockBuilder<'a>,
    pub on_success: LLVMBasicBlockRef,
    pub on_failure: LLVMBasicBlockRef,

    pub sequences: &'a [SequenceBlock],
    pub sequence: usize,
    pub section: usize
}


impl<'a> InstructionBuilder<'a> {
    pub fn build(mut self, instructions: &[Instruction]) {
        let mut append_jump = true;

        for instruction in instructions.iter() {
            match instruction {
                &Instruction::Jump(_, _) => {
                    self.build_advance_section();
                },

                _ => ()
            }

            self.build_instruction(instruction);

            match instruction {
                &Instruction::Exit | &Instruction::SkipIfNotOne | &Instruction::Jump(_, _) 
                    => append_jump = false,

                _ => ()
            }
        }

        if append_jump {
            let section = self.sequences[self.sequence].sections[self.section + 1];
            self.builder.branch(section);
        }
    }


    fn build_advance_section(&mut self) {
        let next_section = self.sequences[self.sequence].jump_table.next_section;
        self.builder.store(i64_value(self.section as i64 + 1), next_section);
    }


    fn build_instruction(&mut self, instruction: &Instruction) {
        match instruction {
            &Instruction::Push(ref source) => self.build_push(source),
            &Instruction::Insert(ref source, ref end) => self.build_insert(source, end),

            &Instruction::OutputCharacter(ref source) => self.build_output_character(source),
            &Instruction::OutputNumber(ref source) => self.build_output_number(source),

            &Instruction::BitwiseNot => self.build_bitwise_not(),
            &Instruction::LogicalNot => self.build_logical_not(),

            &Instruction::Destroy => self.build_destroy(),

            &Instruction::SkipIfNotOne => self.build_skip(),
            &Instruction::Jump(ref direction, ref start) => self.build_jump(direction, start),

            &Instruction::Exit => self.build_exit(),
        }
    }

    fn build_push(&mut self, source: &ValueSource) {
        let value = self.get_value_from_source(source);
        self.builder.call_function("push", &[value]);
    }

    fn build_insert(&mut self, source: &ValueSource, end: &QueueEnd) {
        let value = self.get_value_from_source(source);
        match end {
            &QueueEnd::Front => self.builder.call_function("insert_front", &[value]),
            &QueueEnd::Back => self.builder.call_function("insert_back", &[value]),
        };
    }


    // TODO: remove unneccessary pop/push
    fn build_destroy(&mut self) {
        self.builder.call_function("pop", &[]);
    }


    fn build_output_character(&mut self, source: &ValueSource) {
        let value = self.get_value_from_source(source);
        let value = self.builder.cast_int(value, i32_type());
        self.builder.call_function("putchar", &[value]);
    }

    fn build_output_number(&mut self, source: &ValueSource) {
        let value = self.get_value_from_source(source);
        self.builder.call_function("puti64", &[value]);
    }


    fn build_bitwise_not(&mut self) {
        let value = self.builder.call_function("pop", &[]);
        let value = self.builder.bit_not(value);
        self.builder.call_function("push", &[value]);
    }

    fn build_logical_not(&mut self) {
        let value = self.builder.call_function("pop", &[]);
        let zero = i64_value(0);
        let boolean_value = self.builder.compare(value, Compare::Equal, zero);
        let value = self.builder.zero_extend_int(boolean_value, i64_type());
        self.builder.call_function("push", &[value]);
    }


    fn build_skip(&mut self) {
        let value = self.builder.call_function("pop", &[]);

        let one = i64_value(1);
        let boolean_value = self.builder.compare(value, Compare::Equal, one);

        let zero = i1_value(false);
        let one = i1_value(true);

        let if_next_section = self.sequences[self.sequence].sections[self.section + 1];
        let else_section = self.sequences[self.sequence].sections[self.section + 2];

        self.builder.switch(
            boolean_value,
            &[(one, if_next_section), (zero, else_section)],
            self.on_failure
        );
    }


    fn build_jump(&mut self, direction: &Direction, start: &Start) {
        let target_sequence = match direction {
            &Direction::Previous => self.sequence - 1,
            &Direction::Current => self.sequence,
            &Direction::Next => self.sequence + 1,
        };

        let sequence = &self.sequences[target_sequence];

        match start {
            &Start::Restart => {
                let next_section = sequence.jump_table.next_section;
                self.builder.store(i64_value(0), next_section);
            }
            &Start::Continue => ()
        }

        self.builder.branch(sequence.jump_table.block);
    }


    fn build_exit(&mut self) {
        self.builder.branch(self.on_success);
    }


    fn get_value_from_source(&mut self, source: &ValueSource) -> LLVMValueRef {
        match source {
            &ValueSource::Digit(digit) => i64_value(digit as i64),
            &ValueSource::Pop => self.builder.call_function("pop", &[]),

            &ValueSource::Remove(QueueEnd::Front) => {
                self.builder.call_function("remove_front", &[])
            }
            &ValueSource::Remove(QueueEnd::Back) => {
                self.builder.call_function("remove_back", &[])
            }

            &ValueSource::Operate(ref lhs, ref operation, ref rhs) => {
                let lhs_value = self.get_value_from_source(lhs);
                let rhs_value = self.get_value_from_source(rhs);
                self.build_operation(lhs_value, operation, rhs_value)
            },

            // TODO: remove unneccessary pop/push
            &ValueSource::CloneTop => {
                let value = self.get_value_from_source(&ValueSource::Pop);
                self.builder.call_function("push", &[value]);
                value
            }

            &ValueSource::Input => {
                let value = self.builder.call_function("getchar", &[]);
                self.builder.cast_int(value, i64_type())
            }

            &ValueSource::Equal => {
                let top = self.get_value_from_source(&ValueSource::Pop);
                let front = self.get_value_from_source(&ValueSource::Remove(QueueEnd::Front));
                let comparison = self.builder.compare(top, Compare::Equal, front); 
                self.builder.call_function("insert_front", &[front]);
                self.builder.zero_extend_int(comparison, i64_type())
            }
            &ValueSource::Greater => {
                let top = self.get_value_from_source(&ValueSource::Pop);
                let front = self.get_value_from_source(&ValueSource::Remove(QueueEnd::Front));
                let comparison = self.builder.compare(top, Compare::Greater, front); 
                self.builder.call_function("insert_front", &[front]);
                self.builder.zero_extend_int(comparison, i64_type())
            }
        }
    }

    fn build_operation(&mut self,
                       lhs: LLVMValueRef,
                       op: &Operator,
                       rhs: LLVMValueRef) -> LLVMValueRef {
        match op {
            &Operator::Add => self.builder.add(lhs, rhs),
            &Operator::Sub => self.builder.sub(lhs, rhs),
            &Operator::Mul => self.builder.mul(lhs, rhs),
            &Operator::Div => self.builder.div(lhs, rhs),
            &Operator::Mod => self.builder.modulo(lhs, rhs),

            &Operator::And => self.builder.bit_and(lhs, rhs),
            &Operator::Or => self.builder.bit_or(lhs, rhs),
            &Operator::Xor => self.builder.bit_xor(lhs, rhs),
        }
    }
}
