
extern crate karma_parser;
extern crate llvm_sys;

use karma_parser::*;


#[allow(dead_code)]
mod builder;
use builder::*;

mod stack;
use stack::Stack;

mod deque;
use deque::Deque;

use std::env;
use std::process;

mod sequence_builder;
use sequence_builder::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    let code = if args.len() > 1 {
        let code = parse_file(&args[1]).unwrap();
        optimize::all(code)
    } else {
        println!("Error: no file specified in arguments!");
        process::exit(1);
    };

    let mut builder = Builder::new();

    add_external_functions(&mut builder);
    add_puti64(&mut builder);

    let stack = Stack::build(&mut builder);
    let deque = Deque::build(&mut builder);

    create_main(&mut builder, &stack, &deque, code);


    if builder.is_working() {
        let ir = builder.as_string();
        println!("{}", ir);
    } else {
        process::exit(1);
    }
}


fn add_external_functions(builder: &mut Builder) {
    builder.add_function("malloc", i8_ptr_type(), &[("", i32_type())]);
    builder.add_function("free", void_type(), &[("", i8_ptr_type())]);

    builder.add_function(
        "memcpy", 
        void_type(), 
        &[
            ("", i8_ptr_type()),
            ("", i8_ptr_type()),
            ("", i64_type())
        ]
    );

    builder.add_function("exit", void_type(), &[("", i32_type())]);

    builder.add_function("getchar", i32_type(), &[]);
    builder.add_function("putchar", i32_type(), &[("", i32_type())]);
    builder.add_function_var_arg("printf", i32_type(), &[("", i8_ptr_type())]);
}

fn add_puti64(builder: &mut Builder) {
    let puti64 = builder.add_function("puti64", void_type(), &[("value", i64_type())]);
    let format = builder.constant_string("%ld\0");
    let format = builder.add_global_variable("format", format);
    let value = builder.get_param(puti64, 0);

    let entry = builder.add_block(puti64, "entry");
    builder.build_block(entry, |mut b| {
        let format = b.pointer_cast(format, i8_ptr_type());
        b.call_function("printf", &[format, value]);
        b.return_void();
    });
}

fn create_main(builder: &mut Builder, stack: &Stack, deque: &Deque, sequences: Vec<Sequence>) {
    let main = builder.add_function("main", i32_type(), &mut []);

    let init_stack = builder.add_block(main, "init_stack");
    let init_deque = builder.add_block(main, "init_deque");

    let entry = builder.add_block(main, "entry");
    let exit = builder.add_block(main, "exit");
    let panic = builder.add_block(main, "panic");

    builder.build_block(init_stack, |mut b| {
        stack.build_constructor(&mut b);
        b.branch(init_deque);
    });
    
    builder.build_block(init_deque, |mut b| {
        deque.build_constructor(&mut b);
        b.branch(entry);
    });

    let sequence_blocks = SequenceBuilder::new(builder, main, panic, exit)
        .build(&sequences);

    builder.build_block(entry, |mut b| {
        b.store(i64_value(0), sequence_blocks[1].jump_table.next_section);
        b.branch(sequence_blocks[1].jump_table.block);
    });

    builder.build_block(exit, |mut b| {
        b.return_value(i32_value(0));
    });
    
    builder.build_block(panic, |mut b| {
        b.return_value(i32_value(1));
    });
}


