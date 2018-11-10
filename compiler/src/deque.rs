

use llvm_sys::prelude::*;
use builder::*;
use std::mem::size_of;

const INITIAL_CAPACITY: i64 = 1;

/// A double ended queue with a cyclic buffer
/// ```
/// +---+---+---+---+---+---+---+---+---+
/// | 5 | 3 | 7 | ? | ? | ? | ? | 8 | 1 |
/// +---+---+---+---+---+---+---+---+---+
///               |               |
///               front           back
///
/// front = (back + length) % capacity
/// +---+---+---+---+---+---+---+---+---+
/// | 5 | 3 | 7 | 1 | 4 | 6 | 7 | 8 | 1 |
/// +---+---+---+---+---+---+---+---+---+
///                               |
///                               back
///                               |
///                               front
///
/// ```
#[allow(dead_code)]
pub struct Deque {
    pub data: LLVMValueRef,

    pub back: LLVMValueRef,
    pub length: LLVMValueRef,
    pub capacity: LLVMValueRef,

    pub insert_front: LLVMValueRef,
    pub insert_back: LLVMValueRef,

    pub remove_front: LLVMValueRef,
    pub remove_back: LLVMValueRef,

    resize: LLVMValueRef,
}

impl Deque {
    pub fn build(builder: &mut Builder) -> Deque {
        let data = builder.add_global_variable("deque", i64_ptr_value());

        let back = builder.add_global_variable("deque_back", i64_value(0));
        let length = builder.add_global_variable("deque_length", i64_value(0));
        let capacity = builder.add_global_variable("deque_capacity", i64_value(0));

        let (insert_front, insert_back) = Self::create_insert(builder);
        let (remove_front, remove_back) = Self::create_remove(builder);
        let resize = Self::create_resize(builder);

        let deque = Deque {
            data,
            back,
            length,
            capacity,

            insert_front,
            insert_back,

            remove_front,
            remove_back,

            resize
        };

        deque.build_resize(builder);
        deque.build_insert(builder);
        deque.build_remove(builder);
        
        deque
    }

    pub fn build_constructor(&self, b: &mut BlockBuilder) {
        b.call_function("deque_resize", &[i64_value(INITIAL_CAPACITY)]);
    }

    fn create_resize(builder: &mut Builder) -> LLVMValueRef {
        builder.add_function("deque_resize", void_type(), &[("new_size", i64_type())])
    }

    fn create_insert(builder: &mut Builder) -> (LLVMValueRef, LLVMValueRef) {
        let front = builder.add_function("insert_front", void_type(), &[("value", i64_type())]);
        let back = builder.add_function("insert_back", void_type(), &[("value", i64_type())]);

        (front, back)
    }
    
    fn create_remove(builder: &mut Builder) -> (LLVMValueRef, LLVMValueRef) {
        let front = builder.add_function("remove_front", i64_type(), &[]);
        let back = builder.add_function("remove_back", i64_type(), &[]);

        (front, back)
    }


    // TODO: copy old contents
    fn build_resize(&self, builder: &mut Builder) {
        let new_size = builder.get_param(self.resize, 0);
        let entry = builder.add_block(self.resize, "entry");
        let find_layout = builder.add_block(self.resize, "find_layout");
        let wrapping = builder.add_block(self.resize, "wrapping");
        let linear = builder.add_block(self.resize, "linear");
        let exit = builder.add_block(self.resize, "exit");

        let new_ptr = &mut None;
        let new_size_bytes = &mut None;

        builder.build_block(entry, |mut block| {
            *new_size_bytes = Some(block.mul(new_size, i64_value(size_of::<i64>() as i64)));

            let size = block.cast_int(new_size_bytes.unwrap(), i32_type());
            *new_ptr = Some(block.call_function("malloc", &[size]));

            let capacity = block.load(self.capacity);
            let is_uninitialized = block.compare(capacity, Compare::Equal, i64_value(0));

            block.conditional_branch(is_uninitialized, exit, find_layout);
        });

        builder.build_block(find_layout, |mut block| {
            let back = block.load(self.back);
            let front = self.front(&mut block);
            
            let is_wrapping = block.compare(back, Compare::GreaterOrEqual, front);
            
            block.conditional_branch(is_wrapping, wrapping, linear);
        });

        builder.build_block(linear, |mut block| {
            let capacity = block.load(self.capacity);

            let data = block.load(self.data);
            let data = block.pointer_cast(data, i8_ptr_type());
            
            let old_size_bytes = block.mul(capacity, i64_value(size_of::<i64>() as i64));
            block.call_function("memcpy", 
                                &[
                                    new_ptr.unwrap(),
                                    data,
                                    old_size_bytes,
                                ]);
            block.branch(exit);
        });

        builder.build_block(wrapping, |mut block| {
            let capacity = block.load(self.capacity);
            let back = block.load(self.back);
            let front = self.front(&mut block);

            let data = block.load(self.data);
            let data_ptr = block.pointer_cast(data, i8_ptr_type());

            let first_chunk_size = block.mul(front, i64_value(size_of::<i64>() as i64));
            block.call_function("memcpy", 
                                &[
                                    new_ptr.unwrap(),
                                    data_ptr,
                                    first_chunk_size,
                                ]);

            let distance_to_end = block.sub(capacity, back);
            let last_chunk_size = block.mul(distance_to_end, i64_value(size_of::<i64>() as i64));

            let offset = block.sub(new_size_bytes.unwrap(), last_chunk_size);
            let ptr_offset = block.get_element_offset(new_ptr.unwrap(), offset);

            let data_offset = block.get_element_offset(data, back);
            let data_ptr = block.pointer_cast(data_offset, i8_ptr_type());

            block.call_function("memcpy", 
                                &[
                                    ptr_offset,
                                    data_ptr,
                                    last_chunk_size,
                                ]);

            let new_back = block.sub(new_size, distance_to_end);
            block.store(new_back, self.back);

            block.branch(exit);
        });

        builder.build_block(exit, |mut block| {
            let ptr = block.pointer_cast(new_ptr.unwrap(), i64_ptr_type());
            block.store(ptr, self.data);

            let new_capacity = block.cast_int(new_size, i64_type());
            block.store(new_capacity, self.capacity);

            block.return_void();
        });
    }


    fn create_grow_block(&self,
                           builder: &mut Builder,
                           target_fn: LLVMValueRef,
                           return_block: LLVMBasicBlockRef) -> LLVMBasicBlockRef {
        let resize = builder.add_block(target_fn, "grow");

        builder.build_block(resize, |mut block| {
            let capacity = block.load(self.capacity);
            let new_capacity = block.mul(capacity, i64_value(2));

            block.call(self.resize, &[new_capacity]);
            block.branch(return_block);
        });

        resize
    }


    fn build_insert(&self, builder: &mut Builder) {
        self.build_insert_front(builder);
        self.build_insert_back(builder);
    }

    fn build_remove(&self, builder: &mut Builder) {
        self.build_remove_front(builder);
        self.build_remove_back(builder);
    }


    fn build_insert_front(&self, builder: &mut Builder) {
        let value = builder.get_param(self.insert_front, 0);
        let entry = builder.add_block(self.insert_front, "entry");
        let exit = builder.add_block(self.insert_front, "exit");
        let grow = self.create_grow_block(builder, self.insert_front, exit);

        builder.build_block(entry, |mut block| {
            self.needs_grow(&mut block, grow, exit);
        });

        builder.build_block(exit, |mut block| {
            let front = self.front(&mut block);
            self.write(&mut block, value, front);

            self.add_length(&mut block, 1);

            block.return_void();
        });
    }
    
    
    fn build_insert_back(&self, builder: &mut Builder) {
        let value = builder.get_param(self.insert_back, 0);
        let entry = builder.add_block(self.insert_back, "entry");
        let exit = builder.add_block(self.insert_back, "exit");
        let grow = self.create_grow_block(builder, self.insert_back, exit);

        builder.build_block(entry, |mut block| {
            self.needs_grow(&mut block, grow, exit);
        });

        builder.build_block(exit, |mut block| {
            let back = self.move_back(&mut block, -1);

            self.write(&mut block, value, back);

            block.return_void();
        });
    }


    fn build_remove_front(&self, builder: &mut Builder) {
        let entry = builder.add_block(self.remove_front, "entry");
        let exit = builder.add_block(self.remove_front, "exit");
        let fail = builder.add_block(self.remove_front, "fail");

        builder.build_block(entry, |mut block| {
            self.is_empty(&mut block, fail, exit);
        });

        builder.build_block(exit, |mut block| {
            self.add_length(&mut block, -1);

            let front = self.front(&mut block);
            let value = self.read(&mut block, front);

            block.return_value(value);
        });

        builder.build_block(fail, |mut block| {
            block.call_function("exit", &[i32_value(13)]);
            block.return_value(i64_value(-1));
        });
    }


    fn build_remove_back(&self, builder: &mut Builder) {
        let entry = builder.add_block(self.remove_back, "entry");
        let exit = builder.add_block(self.remove_back, "exit");
        let fail = builder.add_block(self.remove_back, "fail");

        builder.build_block(entry, |mut block| {
            self.is_empty(&mut block, fail, exit);
        });

        builder.build_block(exit, |mut block| {
            let back = block.load(self.back);
            let value = self.read(&mut block, back);

            self.move_back(&mut block, 1);

            block.return_value(value);
        });

        builder.build_block(fail, |mut block| {
            block.call_function("exit", &[i32_value(13)]);
            block.return_value(i64_value(-1));
        });
    }




    fn needs_grow(&self, 
                  block: &mut BlockBuilder, 
                  on_true: LLVMBasicBlockRef,
                  on_false: LLVMBasicBlockRef) {
        let length = block.load(self.length);
        let capacity = block.load(self.capacity);
        let needs_resize = block.compare(length, Compare::GreaterOrEqual, capacity);
        block.conditional_branch(needs_resize, on_true, on_false);
    }

    fn is_empty(&self, 
                block: &mut BlockBuilder, 
                on_true: LLVMBasicBlockRef,
                on_false: LLVMBasicBlockRef) {
        let length = block.load(self.length);
        let is_empty = block.compare(i64_value(0), Compare::Greater, length);
        block.conditional_branch(is_empty, on_true, on_false);
    }


    fn front(&self, block: &mut BlockBuilder) -> LLVMValueRef {
        let back = block.load(self.back);
        let length = block.load(self.length);
        let capacity = block.load(self.capacity);

        let offset = block.add(back, length);

        block.modulo(offset, capacity)
    }


    fn move_back(&self, block: &mut BlockBuilder, direction: i64) -> LLVMValueRef {
        self.add_length(block, -direction);

        let back = block.load(self.back);
        let capacity = block.load(self.capacity);

        let back = block.add(back, i64_value(direction));
        // wrap around
        let back = block.add(back, capacity);
        let back = block.modulo(back, capacity);
        block.store(back, self.back);
        back
    }


    fn add_length(&self, block: &mut BlockBuilder, amount: i64) {
        let length = block.load(self.length);
        let new_length = block.add(length, i64_value(amount));
        block.store(new_length, self.length);
    }


    fn write(&self, builder: &mut BlockBuilder, value: LLVMValueRef, index: LLVMValueRef) {
        let data_ptr = builder.load(self.data);
        let ptr = builder.get_element_offset(data_ptr, index);
        builder.store(value, ptr);
    }
    
    fn read(&self, builder: &mut BlockBuilder, index: LLVMValueRef) -> LLVMValueRef {
        let data_ptr = builder.load(self.data);
        let ptr = builder.get_element_offset(data_ptr, index);
        builder.load(ptr)
    }
}


