
extern crate llvm_sys;


use std::ptr;


mod builder;
use builder::*;

use llvm_sys::prelude::*;

fn main() {
    unsafe {
        // Set up a context, module and builder in that context.
        let mut builder = Builder::new();
        
        add_external_functions(&mut builder);
        create_stack(&mut builder);
        create_main(&mut builder);

        builder.print_module();
        

        /*
        // Get the type signature for void nop(void);
        // Then create it in our module.
        let void = llvm::LLVMVoidTypeInContext(context);
        let function_type = llvm::LLVMFunctionType(void, ptr::null_mut(), 0, 0);
        let function = llvm::LLVMAddFunction(module, b"nop\0".as_ptr() as *const _,
                                                   function_type);

        // Create a basic block in the function and set our builder to generate
        // code in it.
        let bb = llvm::LLVMAppendBasicBlockInContext(context, function,
                                                           b"entry\0".as_ptr() as *const _);
        llvm::LLVMPositionBuilderAtEnd(builder, bb);

        // Emit a `ret void` into the function
        llvm::LLVMBuildRetVoid(builder);

        // Dump the module as IR to stdout.
        llvm::LLVMDumpModule(module);

        // Clean up. Values created in the context mostly get cleaned up there.
        llvm::LLVMDisposeBuilder(builder);
        llvm::LLVMDisposeModule(module);
        llvm::LLVMContextDispose(context);
        */
    }
}


unsafe fn add_external_functions(builder: &mut Builder) {
    builder.add_function("malloc", i8_ptr_type(), &mut [i32_type()]);
    builder.add_function("putchar", i32_type(), &mut [i32_type()]);
}


unsafe fn create_stack(builder: &mut Builder) {
    create_stack_define(builder);
    create_stack_push(builder);
    create_stack_pop(builder);
}

unsafe fn create_main(builder: &mut Builder) {
    let main = builder.add_function("main", i32_type(), &mut []);
    let entry = builder.add_block(main, "entry");

    builder.build_block(entry, |mut b| {
        b.call_function("putchar", &mut [i32_value(072)], "");
        b.call_function("putchar", &mut [i32_value(101)], "");
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

    add_init_stack(builder, entry);
}



unsafe fn create_stack_define(builder: &mut Builder) {
    let data = builder.add_global_variable("stack", i64_ptr_value());
    let head = builder.add_global_variable("head", i64_value(-1));
}

unsafe fn create_stack_push(builder: &mut Builder) {

}

unsafe fn create_stack_pop(builder: &mut Builder) {

}



unsafe fn add_init_stack(builder: &mut Builder, entry: LLVMBasicBlockRef) {
    
}





