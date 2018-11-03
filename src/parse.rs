
use error::*;


#[derive(Debug)]
pub enum Instruction {
    /// Bitwise NOT top of stack
    BitwiseNot,

    /// Logical NOT top of stack
    LogicalNot,


    /// Push value to stack
    Push(ValueSource),

    /// Insert a value into deque
    Insert(ValueSource, QueueEnd),

    /// Destroy the value at top of stack
    Destroy,

    /// Output a unicode character
    OutputCharacter(ValueSource),

    /// Output an integer
    OutputNumber(ValueSource),

    /// Skips the next instruction if top of stack is not 1
    SkipIfNotOne,

    /// Jump to another sequence
    Jump(Direction, Start),

    /// Exits the program
    Exit
}

#[derive(Debug)]
pub enum ValueSource {
    /// A constant digit
    Digit(u8),

    /// Pop a value from the stack
    Pop,

    /// Perform a calculation operation on two values
    Operate(Box<ValueSource>, Operator, Box<ValueSource>),

    /// Get a copy of the top of stack
    CloneTop,

    /// Remove a value from the deque
    Remove(QueueEnd),

    /// Read char from stdin
    Input,

    /// 1 if top of stack == front of deque, 0 otherwise. Pops the top of the stack.
    Equal,

    /// 1 if top of stack > front of deque, 0 otherwise. Pops the top of the stack.
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

    sequences.push(vec![Exit]);

    for sequence in source_code.lines().enumerate().map(|(n, l)| parse_line(l, n + 1)) {
        sequences.push(sequence?)
    }

    sequences.push(vec![Exit]);

    Ok(sequences)
}


fn parse_line(line: &str, line_number: usize) -> Result<Sequence> {
    let mut sequence = Vec::new();

    for character in line.chars() {
        let instruction = match character {
            // Math
            '+' => Push(Operate( Box::new(Pop), Add, Box::new(Pop) )),
            '-' => Push(Operate( Box::new(Pop), Sub, Box::new(Pop) )),
            '*' => Push(Operate( Box::new(Pop), Mul, Box::new(Pop) )),
            '/' => Push(Operate( Box::new(Pop), Div, Box::new(Pop) )),
            '%' => Push(Operate( Box::new(Pop), Mod, Box::new(Pop) )),
            '&' => Push(Operate( Box::new(Pop), And, Box::new(Pop) )),
            '|' => Push(Operate( Box::new(Pop), Or,  Box::new(Pop) )),
            '^' => Push(Operate( Box::new(Pop), Xor, Box::new(Pop) )),

            '~' => BitwiseNot,
            '!' => LogicalNot,

            // Logic
            '=' => Push(Equal),
            '>' => Push(Greater),

            '@' => SkipIfNotOne,

            // Stack/Deque
            digit if digit.is_digit(10) => {
                let value = digit.to_digit(10).unwrap() as u8;
                Push(Digit(value))
            },

            '}' => Insert(Pop, Front),
            '{' => Push(Remove(Front)),
            
            '[' => Insert(Pop, Back),
            ']' => Push(Remove(Back)),

            '#' => Destroy,
            '\\' => Push(CloneTop),

            // IO
            '?' => Push(Input),
            ':' => OutputCharacter(Pop),
            ';' => OutputNumber(Pop),

            // Jumping
            ',' => Jump(Next, Restart),
            '.' => Jump(Next, Continue),
            '\'' => Jump(Previous, Continue),
            '<' => Jump(Current, Restart),

            // Allow whitespace within code
            ' ' => continue,
            '\t' => continue,

            // Ignore a unknown character and anything after it
            _ => break
        };


        sequence.push(instruction);
    }

    match sequence.last() {
        Some(SkipIfNotOne) => return Err(Error::TrailingSkip(line_number)),
        _ => {}
    }

    sequence.push(Exit);
    Ok(sequence)
}
