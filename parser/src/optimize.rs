
use parse::{
    *,
    Instruction::*,
    ValueSource::*,
};


/// Apply all optimizations to the sequence
pub fn all(sequences: Vec<Sequence>) -> Vec<Sequence> {
    compute_constants(sequences)
}


/// Precompute some of the constant values
pub fn compute_constants(sequences: Vec<Sequence>) -> Vec<Sequence> {
    sequences.into_iter().map(|sequence|{
        sequence.into_iter().map(|section| {
            compute_constants_section(section)
        }).collect()
    }).collect()
}


fn compute_constants_section(section: Section) -> Section {
    let mut instructions = Section::new();

    for instruction in section {
        match instruction {
            Push(Operate(ref lhs, ref operator, ref rhs)) 
                if lhs == &Box::new(Pop) && rhs == &Box::new(Pop) => {
                match (instructions.pop(), instructions.pop()) {
                    (Some(Push(Constant(lhs))), Some(Push(Constant(rhs)))) => {
                        let result = compute_constant_operation(lhs, operator, rhs);
                        instructions.push(Push(Constant(result)));
                    }

                    (a, b) => {
                        if let Some(b) = b { instructions.push(b); }
                        if let Some(a) = a { instructions.push(a); }
                        instructions.push(instruction.clone());
                    }
                }
            },

            Push(CloneTop) => {
                match instructions.pop() {
                    Some(push @ Push(Constant(_))) => {
                        instructions.push(push.clone());
                        instructions.push(push);
                    }

                    push => {
                        if let Some(push) = push { instructions.push(push); }
                        instructions.push(instruction);
                    }
                }
            }

            Insert(Pop, end) => {
                match instructions.pop() {
                    Some(Push(constant @ Constant(_))) => {
                        instructions.push(Insert(constant, end));
                    }

                    previous => {
                        if let Some(previous) = previous { instructions.push(previous); }
                        instructions.push(Insert(Pop, end));
                    }
                }
            }

            OutputNumber(Pop) => {
                match instructions.pop() {
                    Some(Push(Constant(value))) => {
                        instructions.push(OutputNumber(Constant(value)));
                    }

                    push => {
                        if let Some(push) = push { instructions.push(push); }
                        instructions.push(instruction);
                    }
                }
            }

            OutputCharacter(Pop) => {
                match instructions.pop() {
                    Some(Push(Constant(value))) => {
                        instructions.push(OutputCharacter(Constant(value)));
                    }

                    push => {
                        if let Some(push) = push { instructions.push(push); }
                        instructions.push(instruction);
                    }
                }
            }

            _ => instructions.push(instruction)
        }
    }

    instructions
}


fn compute_constant_operation(lhs: i64, operator: &Operator, rhs: i64) -> i64 {
    match operator {
        &Operator::Add => lhs + rhs,
        &Operator::Sub => lhs - rhs,
        &Operator::Mul => lhs * rhs,
        &Operator::Div => lhs / rhs,
        &Operator::Mod => lhs % rhs,
        &Operator::And => lhs & rhs,
        &Operator::Or  => lhs | rhs,
        &Operator::Xor => lhs ^ rhs,
    }
}

