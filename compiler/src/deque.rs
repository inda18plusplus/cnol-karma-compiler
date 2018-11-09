

use llvm_sys::prelude::*;
use builder::*;

const DEQUE_SIZE: usize = 100;

#[allow(dead_code)]
pub struct Deque {
    pub data: LLVMValueRef,
    pub front: LLVMValueRef,
    pub back: LLVMValueRef,

    pub insert_front: LLVMValueRef,
    pub insert_back: LLVMValueRef,

    pub remove_front: LLVMValueRef,
    pub remove_back: LLVMValueRef,
}

impl Deque {
    pub fn build(builder: &mut Builder) -> Deque {
        let data = builder.add_global_variable("deque", i64_ptr_value());
        let front = builder.add_global_variable("front", i64_value(DEQUE_SIZE as i64 / 2));
        let back = builder.add_global_variable("back", i64_value(DEQUE_SIZE as i64 / 2));

        let (insert_front, insert_back) = Self::create_insert(builder);
        let (remove_front, remove_back) = Self::create_remove(builder);

        let deque = Deque {
            data,
            front,
            back,

            insert_front,
            insert_back,

            remove_front,
            remove_back
        };

        deque.build_insert(builder);
        deque.build_remove(builder);
        
        deque
    }

    pub fn build_constructor(&self, b: &mut BlockBuilder) {
        use std::mem::size_of;
        let deque_size = DEQUE_SIZE as i32 * size_of::<i64>() as i32;

        let ptr = b.call_function("malloc", &mut[i32_value(deque_size)]);
        let ptr = b.pointer_cast(ptr, i64_ptr_type());
        b.store(ptr, self.data);
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


    // TODO: add fail checks
    // if back >= len then resize/crash
    // if first < len then resize/crash
    fn build_insert(&self, builder: &mut Builder) {
        // front: move then write
        let value = builder.get_param(self.insert_front, 0);
        let entry = builder.add_block(self.insert_front, "entry");
        builder.build_block(entry, |mut b| {
            let pos = b.load(self.front);
            let new_pos = b.add(pos, i64_value(-1));

            b.store(new_pos, self.front);

            self.write(&mut b, new_pos, value);
            b.return_void();
        });

        // back: write then move
        let value = builder.get_param(self.insert_back, 0);
        let entry = builder.add_block(self.insert_back, "entry");
        builder.build_block(entry, |mut b| {
            let pos = b.load(self.back);
            self.write(&mut b, pos, value);

            let new_pos = b.add(pos, i64_value(1));
            b.store(new_pos, self.back);

            b.return_void();
        });
    }

    // TODO: add fail checks
    // if back < first then attempt to remove from empty
    // if back = first then rebalance
    fn build_remove(&self, builder: &mut Builder) {
        // front: read then move
        let entry = builder.add_block(self.remove_front, "entry");
        builder.build_block(entry, |mut b| {
            let pos = b.load(self.front);
            let value = self.read(&mut b, pos);

            let new_pos = b.add(pos, i64_value(1));
            b.store(new_pos, self.front);

            b.return_value(value);
        });

        // back: move then read
        let entry = builder.add_block(self.remove_back, "entry");
        builder.build_block(entry, |mut b| {
            let pos = b.load(self.back);
            let new_pos = b.add(pos, i64_value(-1));
            b.store(new_pos, self.back);

            let value = self.read(&mut b, new_pos);
            b.return_value(value);
        });
    }


    fn write(&self, builder: &mut BlockBuilder, index: LLVMValueRef, value: LLVMValueRef) {
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


