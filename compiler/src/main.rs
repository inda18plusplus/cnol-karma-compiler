
extern crate llvm_sys;
use llvm_sys::prelude::*;

mod builder;
use builder::*;


const STACK_SIZE: usize = 10;


fn main() {
    unsafe {
        let mut builder = Builder::new();
        
        add_external_functions(&mut builder);

        let stack = Stack::build(&mut builder);

        create_main(&mut builder, &stack);

        builder.print_module();
    }
}


unsafe fn add_external_functions(builder: &mut Builder) {
    builder.add_function("malloc", i8_ptr_type(), &[("", i32_type())]);
    builder.add_function("putchar", i32_type(), &[("", i32_type())]);
}


unsafe fn create_main(builder: &mut Builder, stack: &Stack) {
    let main = builder.add_function("main", i32_type(), &mut []);
    let init_stack = builder.add_block(main, "init_stack");
    let entry = builder.add_block(main, "entry");

    builder.build_block(init_stack, |mut b| {
        stack.build_constructor(&mut b, stack.data);
        b.branch(entry);
    });

    builder.build_block(entry, |mut b| {
        b.call_function("push", &mut [i64_value(072)], "");
        b.call_function("push", &mut [i64_value(101)], "");

        let e = b.call_function("pop", &mut [], "");
        let h = b.call_function("pop", &mut [], "");

        let e = b.cast_int(e, i32_type(), "");
        let h = b.cast_int(h, i32_type(), "");

        b.call_function("putchar", &mut [h], "");
        b.call_function("putchar", &mut [e], "");
        b.call_function("putchar", &mut [i32_value(108)], "");
        b.call_function("putchar", &mut [i32_value(108)], "");
        b.call_function("putchar", &mut [i32_value(111)], "");
        b.call_function("putchar", &mut [i32_value(044)], "");
        b.call_function("putchar", &mut [i32_value(032)], "");
        b.call_function("putchar", &mut [i32_value(119)], "");
        b.call_function("putchar", &mut [i32_value(111)], "");
        b.call_function("putchar", &mut [i32_value(114)], "");
        b.call_function("putchar", &mut [i32_value(108)], "");
        b.call_function("putchar", &mut [i32_value(100)], "");
        b.call_function("putchar", &mut [i32_value(033)], "");
        b.call_function("putchar", &mut [i32_value(010)], "");
        b.return_value(i32_value(0));
    });
}


#[allow(dead_code)]
struct Stack {
    data: LLVMValueRef,
    head: LLVMValueRef,

    push: LLVMValueRef,
    pop: LLVMValueRef
}

impl Stack {
    pub unsafe fn build(builder: &mut Builder) -> Stack {
        let data = builder.add_global_variable("stack", i64_ptr_value());
        let head = builder.add_global_variable("head", i64_value(-1));
        let push = Stack::build_push(builder, data, head);
        let pop = Stack::build_pop(builder, data, head);

        Stack {
            data,
            head,
            push,
            pop
        }
    }

    unsafe fn build_push(builder: &mut Builder, 
                         data: LLVMValueRef, 
                         head: LLVMValueRef) -> LLVMValueRef {
        let push = builder.add_function("push", void_type(), &[("value", i64_type())]);
        let value = builder.get_param(push, 0);

        let body = builder.add_block(push, "entry");

        builder.build_block(body, |mut b| {
            let b = &mut b;
            let pos = Self::move_head(b, head, i64_value(1));
            let data_ptr = b.load(data, "");
            let ptr = b.get_element_offset(data_ptr, pos, "");
            b.store(value, ptr);

            b.return_void();
        });

        push
    }

    unsafe fn build_pop(builder: &mut Builder, 
                        data: LLVMValueRef, 
                        head: LLVMValueRef) -> LLVMValueRef {
        let pop = builder.add_function("pop", i64_type(), &[]);
        let body = builder.add_block(pop, "entry");

        builder.build_block(body, |mut b| {
            let b = &mut b;
            let pos = b.load(head, "");
            let data_ptr = b.load(data, "");
            let ptr = b.get_element_offset(data_ptr, pos, "");
            let value = b.load(ptr, "");
            
            Self::move_head(b, head, i64_value(-1));

            b.return_value(value);
        });

        pop
    }


    unsafe fn build_constructor(&self, b: &mut BlockBuilder, data: LLVMValueRef) {
        let ptr = b.call_function("malloc", &mut[i32_value(STACK_SIZE as i32)], "");
        let ptr = b.pointer_cast(ptr, i64_ptr_type(), "");
        b.store(ptr, data);
    }


    /// Moves the head of the stack forward or backwards and returns it's position after the move
    unsafe fn move_head(b: &mut BlockBuilder, head: LLVMValueRef, delta: LLVMValueRef) -> LLVMValueRef {
        let old_head = b.load(head, "");
        let new_head = b.add(old_head, delta, "");
        b.store(new_head, head);
        new_head
    }
}




