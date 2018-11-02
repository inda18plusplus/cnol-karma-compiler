
use error::*;


#[derive(Debug)]
pub enum Instruction {
    /// Bitwise NOT top of stack
    BitNot,

    /// Logical NOT top of stack
    LogNot,

    /// Perform an operation on two values
    Operate { 
        operator: Operator, 
        lhs: ValueSource, 
        rhs: ValueSource, 
    },

    /// Push value to stack
    Push(ValueSource),

    /// Insert a value into deque
    Insert(QueueEnd),

    /// Destroy the value at top of stack
    Destroy,

    /// Output a unicode character
    OutputCharacter(ValueSource),

    /// Output an integer
    OutputNumber(ValueSource),

    /// Skips the next instruction if top of stack is not 1
    SkipIfNotOne,

    /// Jump to another sequence
    Jump { direction: Direction, start: Start }
}

#[derive(Debug)]
pub enum ValueSource {
    /// A constant digit
    Digit(u8),

    /// Pop a value from the stack
    Pop,

    /// Get a copy of the top of stack
    CloneTop,

    /// Remove a value from the deque
    Remove(QueueEnd),

    /// Read char from stdin
    Input,

    /// 1 if top of stack == front of deque, 0 otherwise
    Eq,

    /// 1 if top of stack > front of deque, 0 otherwise
    Greater,
}

/// Logical and mathematical operations between values
#[derive(Debug)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    /// Bitwise AND
    And,

    /// Bitwise OR
    Or,

    /// Bitwise XOR
    Xor,
}


#[derive(Debug)]
pub enum QueueEnd {
    Front,
    Back
}

#[derive(Debug)]
pub enum Direction {
    Current,
    Previous,
    Next
}

#[derive(Debug)]
pub enum Start {
    /// Execute sequence from the beginning
    Restart,

    /// Continue executing sequence where it last left off
    Continue
}

pub type Sequence = Vec<Instruction>;


use self::Instruction::*;
use self::ValueSource::*;
use self::Operator::*;
use self::QueueEnd::*;
use self::Direction::*;
use self::Start::*;


/// Parse the source code into multiple sequences of commands
pub fn interpret(source_code: &str) -> Result<Vec<Sequence>> {
    let mut sequences = Vec::new();

    for sequence in source_code.lines().map(parse_line) {
        sequences.push(sequence?)
    }

    Ok(sequences)
}


fn parse_line(line: &str) -> Result<Sequence> {
    let mut sequence = Vec::new();

    for character in line.chars() {
        let instruction = match character {
            // [0-9]
            digit if digit.is_digit(10) => {
                let value = digit.to_digit(10).unwrap() as u8;
                Push(Digit(value))
            },

            '?' => Push(Input),
            ':' => OutputCharacter(Pop),
            ';' => OutputNumber(Pop),

            command => return Err(Error::InvalidCommand(command))
        };


        sequence.push(instruction);
    }

    Ok(sequence)
}

