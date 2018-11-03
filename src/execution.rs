

use std::{
    collections::VecDeque,

    io::{
        self,
        Read
    }
};

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
        state.advance();

        match instruction {
            &BitwiseNot => {
                let value = state.pop();
                state.push(!value)
            }

            &LogicalNot => {
                let value = state.pop();
                state.push(if value == 0 {1} else {0});
            }


            &Push(ref source) => {
                let value = state.value_from_source(source);
                state.push(value);
            }

            &Destroy => {
                state.pop();
            }

            &Insert(ref source, ref end) => {
                let value = state.value_from_source(source);
                state.insert(value, end);
            }


            &SkipIfNotOne => {
                let top = state.pop();
                if top != 1 {
                    state.advance();
                }
                state.push(top);
            }

            &Jump(ref direction, ref start) => {
                state.jump(direction, start);
            }


            &OutputNumber(ref source) => {
                let value = state.value_from_source(source);
                print!("{}", value);
            }

            &OutputCharacter(ref source) => {
                let value = state.value_from_source(source);
                print!("{}", value as u8 as char);
            }

        }
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

    pub fn pop(&mut self) -> DataType {
        self.stack.pop().unwrap()
    }
    
    pub fn insert(&mut self, value: DataType, end: &QueueEnd) {
        match end {
            &Back => self.deque.push_back(value),
            &Front => self.deque.push_front(value)
        }
    }

    pub fn remove(&mut self, end: &QueueEnd) -> DataType {
        match end {
            &Back => self.deque.pop_back().unwrap(),
            &Front => self.deque.pop_front().unwrap(),
        }
    }

    pub fn jump(&mut self, direction: &Direction, start: &Start) {
        match direction {
            &Previous => self.current_sequence -= 1,
            &Current => {},
            &Next => self.current_sequence += 1
        }

        match start {
            &Restart => self.instruction_offsets[self.current_sequence] = 0,
            &Continue => {}
        }
    }

    pub fn value_from_source(&mut self, source: &ValueSource) -> DataType {
        match source {
            &Pop => self.stack.pop().unwrap(),

            &Remove(ref end) => {
                self.remove(end)
            }

            
            &Digit(ref value) => *value as DataType,

            &Operate(ref lhs, ref operator, ref rhs) => {
                let lhs_value = self.value_from_source(lhs);
                let rhs_value = self.value_from_source(rhs);
                perform_operation(lhs_value, rhs_value, operator)
            }

            &CloneTop => {
                self.stack.last().unwrap().clone()
            }

            &Input => {
                let stdin = io::stdin();
                stdin.bytes().next().unwrap().unwrap().clone() as DataType
            }


            &Equal => {
                let top = self.stack.last().unwrap();
                let front = self.deque.front().unwrap();
                (top == front) as DataType
            }

            &Greater => {
                let top = self.stack.last().unwrap();
                let front = self.deque.front().unwrap();
                (top > front) as DataType
            }
        }
    }
}

fn perform_operation(lhs: DataType, rhs: DataType, operator: &Operator) -> DataType {
    match operator {
        &Add => lhs + rhs,
        &Sub => lhs - rhs,
        &Mul => lhs * rhs,
        &Div => lhs / rhs,
        &Mod => lhs % rhs,
        &And => lhs & rhs,
        &Or  => lhs | rhs,
        &Xor => lhs ^ rhs,
    }
}
