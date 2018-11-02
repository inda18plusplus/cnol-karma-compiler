

use std::collections::VecDeque;

use error::*;
use parse::*;

use parse::Instruction::*;
use parse::ValueSource::*;
use parse::Operator::*;
use parse::QueueEnd::*;
use parse::Direction::*;
use parse::Start::*;

pub type DataType = i64;
type Stack = Vec<DataType>;
type Deque = VecDeque<DataType>;


struct State {
    pub stack: Stack,
    pub deque: Deque,

    pub current_sequence: usize,
    pub instruction_offsets: Vec<usize>
}



pub fn execute(sequences: &[Sequence]) -> Result<()> {
    let mut state = State {
        stack: Stack::new(),
        deque: Deque::new(),

        current_sequence: 0,
        instruction_offsets: vec![0; sequences.len()]
    };

    while let Some(instruction) = state.get_instruction(sequences) {
        match instruction {
            &Push(ref source) => {
                let value = state.value_from_source(source);
                state.push(value);
            }

            &OutputNumber(ref source) => {
                let value = state.value_from_source(source);
                print!("{}", value);
            }

            instruction => unimplemented!("{:?}", instruction)
        }

        state.advance();
    }

    Ok(())
}



impl State {
    pub fn get_instruction<'a>(&self, sequences: &'a [Sequence]) -> Option<&'a Instruction> {
        let sequence = &sequences[self.current_sequence];
        let current_instruction = self.instruction_offsets[self.current_sequence];

        sequence.get(current_instruction)
    }
    
    pub fn advance(&mut self) {
        self.instruction_offsets[self.current_sequence] += 1;
    }


    pub fn push(&mut self, value: DataType) {
        self.stack.push(value);
    }
    
    pub fn insert(&mut self, value: DataType, end: QueueEnd) {
        match end {
            Back => self.deque.push_back(value),
            Front => self.deque.push_front(value)
        }
    }

    pub fn value_from_source(&mut self, source: &ValueSource) -> DataType {
        match source {
            Pop => self.stack.pop().unwrap(),
            Digit(value) => *value as DataType,

            source => unimplemented!("{:?}", source)
        }
    }
}



