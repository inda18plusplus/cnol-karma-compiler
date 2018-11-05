
extern crate karma_parser;
extern crate llvm_sys;

use llvm_sys::prelude::*;

use karma_parser::*;


#[allow(dead_code)]
mod builder;
use builder::*;

mod stack;
use stack::Stack;

use std::env;
use std::process;



fn main() {
    let args: Vec<String> = env::args().collect();

    let code = if args.len() > 1 {
        parse_file(&args[1]).unwrap()
    } else {
        println!("Error: no file specified in arguments!");
        process::exit(1);
    };

    unsafe {
        let mut builder = Builder::new();

        add_external_functions(&mut builder);
        add_puti64(&mut builder);

        let stack = Stack::build(&mut builder);

        create_main(&mut builder, &stack, code);

        builder.print_module();
    }
}


unsafe fn add_external_functions(builder: &mut Builder) {
    builder.add_function("malloc", i8_ptr_type(), &[("", i32_type())]);
    builder.add_function("getchar", i32_type(), &[]);
    builder.add_function_var_arg("printf", i32_type(), &[("", i8_ptr_type())]);
}

unsafe fn add_puti64(builder: &mut Builder) {
    let puti64 = builder.add_function("puti64", void_type(), &[("value", i64_type())]);
    let format = builder.constant_string("%ld\0");
    let format = builder.add_global_variable("format", format);
    let value = builder.get_param(puti64, 0);

    let entry = builder.add_block(puti64, "entry");
    builder.build_block(entry, |mut b| {
        let format = b.pointer_cast(format, i8_ptr_type(), "");
        b.call_function("printf", &[format, value], "");
        b.return_void();
    });
}

unsafe fn create_main(builder: &mut Builder, stack: &Stack, sequences: Vec<Sequence>) {
    let main = builder.add_function("main", i32_type(), &mut []);
    let init_stack = builder.add_block(main, "init_stack");
    let entry = builder.add_block(main, "entry");
    let exit = builder.add_block(main, "exit");

    builder.build_block(init_stack, |mut b| {
        stack.build_constructor(&mut b, stack.data);
        b.branch(entry);
    });

    let sequence_blocks = create_sequence_blocks(builder, main, exit, sequences);

    builder.build_block(entry, |mut b| {
        b.store(i64_value(0), sequence_blocks[1].next_section);
        b.branch(sequence_blocks[1].jump);
    });

    builder.build_block(exit, |mut b| {
        b.return_value(i32_value(1));
    });
}



unsafe fn create_sequence_blocks(builder: &mut Builder,
                                 target_fn: LLVMValueRef,
                                 exit_block: LLVMBasicBlockRef,
                                 sequences: Vec<Sequence>) -> Vec<SequenceBlock> {
    let mut sequence_blocks = Vec::new();

    for (i, sequence) in sequences.iter().enumerate() {
        let jump = builder.add_block(target_fn, &format!("jump_table_{}", i));
        let next_section = builder.add_global_variable(&format!("next_section_{}", i), i64_value(0));
        let sections = create_section_blocks(builder, target_fn, i, next_section, sequence);

        builder.build_block(jump, |mut b| {
            build_jump_table(&mut b, sections, exit_block, next_section);
        });

        sequence_blocks.push(SequenceBlock {
            jump, next_section
        });
    }

    sequence_blocks
}


struct SequenceBlock {
    // the jump table into the sequence
    jump: LLVMBasicBlockRef,

    // i64*, the next block to execute in the sequence
    next_section: LLVMValueRef
}


unsafe fn create_section_blocks(builder: &mut Builder,
                                target_fn: LLVMValueRef,
                                sequence_number: usize,
                                next_section: LLVMValueRef,
                                sequence: &Sequence) -> Vec<LLVMBasicBlockRef> {
    let mut sections = Vec::new();

    let add_section = |builder: &mut Builder, sections: &mut Vec<_>| -> LLVMBasicBlockRef {
        let section_number = sections.len();
        let name = &format!("section_{}_{}", sequence_number, section_number);
        let section = builder.add_block(target_fn, name);

        builder.build_block(section, |mut b| {
            b.store(i64_value(section_number as i64 + 1), next_section);
        });

        sections.push(section);
        section
    };

    let mut current_section = add_section(builder, &mut sections);

    for instruction in sequence.iter() {
        add_instruction(builder, current_section, instruction);
    }

    sections
}

unsafe fn add_instruction(builder: &mut Builder,
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


unsafe fn add_push(builder: &mut Builder,
                   section: LLVMBasicBlockRef,
                   source: &ValueSource) {
    builder.build_block(section, |mut b| {
        let value = get_value_from_source(&mut b, source);
        b.call_function("push", &[value], "");
    })
}

unsafe fn add_output_character(builder: &mut Builder,
                               section: LLVMBasicBlockRef,
                               source: &ValueSource) {
    builder.build_block(section, |mut b| {
        let value = get_value_from_source(&mut b, source);
        let value = b.cast_int(value, i32_type(), "");
        b.call_function("putchar", &[value], "");
    })
}

unsafe fn add_output_number(builder: &mut Builder,
                            section: LLVMBasicBlockRef,
                            source: &ValueSource) {
    builder.build_block(section, |mut b| {
        let value = get_value_from_source(&mut b, source);
        b.call_function("puti64", &[value], "");
    })
}

unsafe fn add_exit(builder: &mut Builder,
                   section: LLVMBasicBlockRef) {
    builder.build_block(section, |mut b| {
        b.return_value(i32_value(0));
    })
}


unsafe fn get_value_from_source(builder: &mut BlockBuilder,
                                source: &ValueSource) -> LLVMValueRef {
    match source {
        &ValueSource::Digit(digit) => i64_value(digit as i64),
        &ValueSource::Pop => builder.call_function("pop", &[], ""),

        _ => unimplemented!()
    }
}


unsafe fn build_jump_table(builder: &mut BlockBuilder,
                           sections: Vec<LLVMBasicBlockRef>,
                           exit: LLVMBasicBlockRef,
                           next_section: LLVMValueRef) {
    let next = builder.load(next_section, "");
    let numbered_sections: Vec<_> = sections.into_iter().enumerate()
        .map(|(i, section)| (i64_value(i as i64), section))
        .collect();

    builder.switch(next, &numbered_sections, exit);
}
