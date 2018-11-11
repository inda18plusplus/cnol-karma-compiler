
use llvm_sys::prelude::*;
use builder::*;

use std::mem::size_of;

const INITIAL_SIZE: i64 = 16;

#[allow(dead_code)]
pub struct Stack {
    pub data: LLVMValueRef,
    pub length: LLVMValueRef,
    pub capacity: LLVMValueRef,

    pub resize: LLVMValueRef,
    
    pub push: LLVMValueRef,
    pub pop: LLVMValueRef
}

impl Stack {
    pub fn build(builder: &mut Builder) -> Stack {
        let data = builder.add_global_variable("stack", i64_ptr_value());
        let length = builder.add_global_variable("stack_length", i64_value(0));
        let capacity = builder.add_global_variable("stack_capacity", i64_value(0));

        let resize = Stack::create_resize(builder);
        let push = Stack::create_push(builder);
        let pop = Stack::create_pop(builder);

        let stack = Stack {
            data,
            length,
            capacity,

            resize,
            push,
            pop
        };

        stack.build_resize(builder);
        stack.build_push(builder);
        stack.build_pop(builder);
        
        stack
    }

    fn create_resize(builder: &mut Builder) -> LLVMValueRef {
        builder.add_function("stack_resize", void_type(), &[("new_size", i64_type())])
    }

    fn create_push(builder: &mut Builder) -> LLVMValueRef {
        builder.add_function("push", void_type(), &[("value", i64_type())])
    }

    fn create_pop(builder: &mut Builder) -> LLVMValueRef {
        builder.add_function("pop", i64_type(), &[])
    }


    fn build_resize(&self, builder: &mut Builder) {
        let new_size = builder.get_param(self.resize, 0);
        let entry = builder.add_block(self.resize, "entry");

        builder.build_block(entry, |mut block| {
            let capacity = block.load(self.capacity);
            let new_size = block.cast_int(new_size, i32_type());

            let new_size_bytes = block.mul(new_size, i32_value(size_of::<i64>() as i32));
            let old_size_bytes = block.mul(capacity, i64_value(size_of::<i64>() as i64));

            let ptr = block.call_function("malloc", &[new_size_bytes]);

            let data = block.load(self.data);
            let data = block.pointer_cast(data, i8_ptr_type());
            
            block.call_function("memcpy", 
                                &[
                                    ptr,
                                    data,
                                    old_size_bytes,
                                ]);

            block.call_function("free", &[data]);
            
            let ptr = block.pointer_cast(ptr, i64_ptr_type());
            block.store(ptr, self.data);


            let new_capacity = block.cast_int(new_size, i64_type());
            block.store(new_capacity, self.capacity);

            block.return_void();
        });
    }

    fn build_push(&self, builder: &mut Builder) {
        let value = builder.get_param(self.push, 0);
        let entry = builder.add_block(self.push, "entry");
        let grow = builder.add_block(self.push, "grow");
        let write = builder.add_block(self.push, "write");

        builder.build_block(entry, |mut block| {
            let length = block.load(self.length);
            let capacity = block.load(self.capacity);
            let new_length = block.add(length, i64_value(1));

            block.store(new_length, self.length);

            let needs_resize = block.compare(new_length, Compare::Greater, capacity);
            block.conditional_branch(needs_resize, grow, write);
        });

        builder.build_block(grow, |mut block| {
            let capacity = block.load(self.capacity);
            let new_capacity = block.mul(capacity, i64_value(2));

            block.call(self.resize, &[new_capacity]);
            block.branch(write);
        });

        builder.build_block(write, |mut block| {
            let length = block.load(self.length);
            let index = block.sub(length, i64_value(1));

            let data_ptr = block.load(self.data);
            let element = block.get_element_offset(data_ptr, index);
            block.store(value, element);

            block.return_void();
        });
    }

    fn build_pop(&self, builder: &mut Builder) {
        let entry = builder.add_block(self.pop, "entry");
        let fail = builder.add_block(self.pop, "fail");
        let read = builder.add_block(self.pop, "read");

        builder.build_block(entry, |mut block| {
            let length = block.load(self.length);
            let new_length = block.sub(length, i64_value(1));

            block.store(new_length, self.length);

            let was_empty = block.compare(i64_value(0), Compare::Greater, new_length);
            block.conditional_branch(was_empty, fail, read);
        });

        builder.build_block(fail, |mut block| {
            block.call_function("exit", &[i32_value(14)]);
            block.return_value(i64_value(-1));
        });

        builder.build_block(read, |mut block| {
            let index = block.load(self.length);

            let data_ptr = block.load(self.data);
            let element = block.get_element_offset(data_ptr, index);
            let value = block.load(element);

            block.return_value(value);
        });
    }


    pub fn build_constructor(&self, b: &mut BlockBuilder) {
        b.call(self.resize, &mut[i64_value(INITIAL_SIZE)]);
    }
}

