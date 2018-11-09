
use llvm_sys::prelude::*;
use builder::*;

const STACK_SIZE: usize = 100;

#[allow(dead_code)]
pub struct Stack {
    pub data: LLVMValueRef,
    pub head: LLVMValueRef,

    pub push: LLVMValueRef,
    pub pop: LLVMValueRef
}

impl Stack {
    pub fn build(builder: &mut Builder) -> Stack {
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

    fn build_push(builder: &mut Builder, 
                  data: LLVMValueRef, 
                  head: LLVMValueRef) -> LLVMValueRef {
        let push = builder.add_function("push", void_type(), &[("value", i64_type())]);
        let value = builder.get_param(push, 0);

        let body = builder.add_block(push, "entry");

        builder.build_block(body, |mut b| {
            let b = &mut b;
            let pos = Self::move_head(b, head, i64_value(1));
            let data_ptr = b.load(data);
            let ptr = b.get_element_offset(data_ptr, pos);
            b.store(value, ptr);

            b.return_void();
        });

        push
    }

    fn build_pop(builder: &mut Builder, 
                 data: LLVMValueRef, 
                 head: LLVMValueRef) -> LLVMValueRef {
        let pop = builder.add_function("pop", i64_type(), &[]);
        let body = builder.add_block(pop, "entry");

        builder.build_block(body, |mut b| {
            let b = &mut b;
            let pos = b.load(head);
            let data_ptr = b.load(data);
            let ptr = b.get_element_offset(data_ptr, pos);
            let value = b.load(ptr);

            Self::move_head(b, head, i64_value(-1));

            b.return_value(value);
        });

        pop
    }


    pub fn build_constructor(&self, b: &mut BlockBuilder) {
        let ptr = b.call_function("malloc", &mut[i32_value(STACK_SIZE as i32)]);
        let ptr = b.pointer_cast(ptr, i64_ptr_type());
        b.store(ptr, self.data);
    }


    /// Moves the head of the stack forward or backwards and returns it's position after the move
    fn move_head(b: &mut BlockBuilder, head: LLVMValueRef, delta: LLVMValueRef) -> LLVMValueRef {
        let old_head = b.load(head);
        let new_head = b.add(old_head, delta);
        b.store(new_head, head);
        new_head
    }
}

