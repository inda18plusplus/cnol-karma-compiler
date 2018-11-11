
use error::*;
use std;


#[derive(Debug, Eq, PartialEq, Clone)]
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

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ValueSource {
    /// A constant value
    Constant(i64),

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
#[derive(Debug, Eq, PartialEq, Clone)]
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


#[derive(Debug, Eq, PartialEq, Clone)]
pub enum QueueEnd {
    Front,
    Back
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Direction {
    Current,
    Previous,
    Next
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Start {
    /// Execute sequence from the beginning
    Restart,

    /// Continue executing sequence where it last left off
    Continue
}

/// Represents a single line in the Karma source
pub type Sequence = Vec<Section>;

/// Represents a block between jumps in the Karma source
pub type Section = Vec<Instruction>;


enum ControlFlow {
    Continue,
    Break
}


impl Instruction {
    fn from(character: char) -> std::result::Result<Instruction, ControlFlow> {
        match character {
            // Math
            '+' => Ok(Push(Operate( Box::new(Pop), Add, Box::new(Pop) ))),
            '-' => Ok(Push(Operate( Box::new(Pop), Sub, Box::new(Pop) ))),
            '*' => Ok(Push(Operate( Box::new(Pop), Mul, Box::new(Pop) ))),
            '/' => Ok(Push(Operate( Box::new(Pop), Div, Box::new(Pop) ))),
            '%' => Ok(Push(Operate( Box::new(Pop), Mod, Box::new(Pop) ))),
            '&' => Ok(Push(Operate( Box::new(Pop), And, Box::new(Pop) ))),
            '|' => Ok(Push(Operate( Box::new(Pop), Or,  Box::new(Pop) ))),
            '^' => Ok(Push(Operate( Box::new(Pop), Xor, Box::new(Pop) ))),

            '~' => Ok(BitwiseNot),
            '!' => Ok(LogicalNot),

            // Logic
            '=' => Ok(Push(Equal)),
            '>' => Ok(Push(Greater)),

            '@' => Ok(SkipIfNotOne),

            // Stack/Deque
            digit if digit.is_digit(10) => {
                let value = digit.to_digit(10).unwrap() as i64;
                Ok(Push(Constant(value)))
            },

            '}' => Ok(Insert(Pop, Front)),
            '{' => Ok(Push(Remove(Front))),
            
            '[' => Ok(Insert(Pop, Back)),
            ']' => Ok(Push(Remove(Back))),

            '#' => Ok(Destroy),
            '\\' => Ok(Push(CloneTop)),

            // IO
            '?' => Ok(Push(Input)),
            ':' => Ok(OutputCharacter(Pop)),
            ';' => Ok(OutputNumber(Pop)),

            // Jumping
            ',' => Ok(Jump(Next, Restart)),
            '.' => Ok(Jump(Next, Continue)),
            '\'' => Ok(Jump(Previous, Continue)),
            '<' => Ok(Jump(Current, Restart)),

            // Allow whitespace within code
            ' ' | '\t' => Err(ControlFlow::Continue),

            // Ignore a unknown character and anything after it
            _ => Err(ControlFlow::Break)
        }
    }

    fn breaks_section(&self) -> bool {
        match self {
            &Instruction::Jump(_, _) | &Instruction::SkipIfNotOne => true,
            _ => false
        }
    }
}




use self::Instruction::*;
use self::ValueSource::*;
use self::Operator::*;
use self::QueueEnd::*;
use self::Direction::*;
use self::Start::*;


/// Parse the source code into multiple sequences of commands
pub fn parse_str(source_code: &str) -> Result<Vec<Sequence>> {
    let mut sequences = Vec::new();

    sequences.push(vec![vec![Exit]]);

    for sequence in source_code.lines().take_while(|line| !line.is_empty()).enumerate()
        .map(|(n, l)| parse_line(&mut l.chars(), n + 1)) {
        sequences.push(sequence?)
    }

    sequences.push(vec![vec![Exit]]);

    Ok(sequences)
}


fn parse_line(characters: &mut impl Iterator<Item=char>, line_number: usize) -> Result<Sequence> {
    let mut sequence = Vec::new();
    let mut section = Vec::new();

    while let Some(character) = characters.next() {
        let instruction = Instruction::from(character);

        match instruction {
            Err(ControlFlow::Break) => break,
            Err(ControlFlow::Continue) => (),

            Ok(instruction @ Instruction::SkipIfNotOne) => {
                section.push(instruction);

                sequence.push(section);

                let mut following_sections = parse_line(characters, line_number).unwrap();
                let if_one = following_sections[0].remove(0);
                
                if following_sections[0].len() == 0 {
                    following_sections.remove(0);
                }
                
                if following_sections.len() == 0 {
                    return Err(Error::TrailingSkip(line_number));
                }

                sequence.push(vec![if_one]);
                sequence.extend(following_sections);

                return Ok(sequence);
            }

            Ok(instruction) => {
                let new_section = instruction.breaks_section();

                section.push(instruction);
                if new_section {
                    sequence.push(section);
                    section = Vec::new();
                }
            }
        }
    }

    section.push(Exit);
    sequence.push(section);

    Ok(sequence)
}

