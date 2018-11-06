

use std::{
    collections::VecDeque,

    io::{
        self,
        Read
    },
    slice
};

use karma_parser::*;
use karma_parser::Instruction::*;
use karma_parser::ValueSource::*;
use karma_parser::Operator::*;
use karma_parser::QueueEnd::*;
use karma_parser::Direction::*;
use karma_parser::Start::*;

pub type DataType = i64;
type Stack = Vec<DataType>;
type Deque = VecDeque<DataType>;


pub fn execute(sequences: &[Sequence]) {
    #[cfg(feature = "debug")]
    eprintln!("Sequences: {:#?}", sequences);

    let mut state = State::new(sequences);
    while let Some(instruction) = state.next_instruction() {
        #[cfg(feature = "debug")]
        {
            let pad = |len, mut string: String| {
                for _ in string.len()..len {
                    string += " ";
                }

                string
            };

            let instr = pad(50, format!("{:?}", instruction));
            let stack = pad(50, format!("{:?}", state.stack));

            eprintln!("{} {} {:?}", instr, stack, state.deque);
        }

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
                    state.next_instruction();
                }
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

            &Exit => break
        }
    }

    #[cfg(feature = "debug")]
    {
        eprintln!("");
        eprintln!("");
        eprintln!("Final stack: {:?}", state.stack);
        eprintln!("Final deque: {:?}", state.deque);
    }
}


#[derive(Debug)]
struct State<'a> {
    stack: Stack,
    deque: Deque,

    current_sequence: usize,
    next_sections: Vec<usize>,
    current_section: slice::Iter<'a, Instruction>,

    sequences: &'a [Sequence]
}

impl<'a> State<'a> {
    pub fn new(sequences: &'a[Sequence]) -> Self {
        let current_section = sequences[1][0].iter();
        let mut next_sections = vec![0; sequences.len()];
        next_sections[1] = 1;

        State {
            stack: Stack::new(),
            deque: Deque::new(),

            current_sequence: 0,
            next_sections,
            current_section,

            sequences
        }
    }


    pub fn next_instruction(&mut self) -> Option<&'a Instruction> {
        match self.current_section.next() {
            instruction @ Some(_) => instruction,

            // go to following section
            None => {
                let next_section = self.next_sections[self.current_sequence];
                if next_section >= self.sequences[self.current_sequence].len() {
                    None
                } else {
                    self.current_section =
                        self.sequences[self.current_sequence][next_section].iter();

                    self.next_sections[self.current_sequence] += 1;
                    self.next_instruction()
                }
            }
        }
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
            &Restart => self.next_sections[self.current_sequence] = 0,
            &Continue => {}
        }

        let next_section = &mut self.next_sections[self.current_sequence];
        self.current_section = self.sequences[self.current_sequence][*next_section].iter();
        *next_section += 1;
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
                let top = self.pop();
                let front = self.deque.front().unwrap();
                (top == *front) as DataType
            }

            &Greater => {
                let top = self.pop();
                let front = self.deque.front().unwrap();
                (top > *front) as DataType
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
