
use llvm_sys::{
    core as llvm,
    prelude::*
};

use std::ffi::CString;

pub struct Builder {
    context: LLVMContextRef,
    module: LLVMModuleRef,

    // Take ownership of strings until module is disposed
    strings: Vec<CString>
}

impl Builder {
    /// Create a new builder
    pub unsafe fn new() -> Builder {
        let context = llvm::LLVMContextCreate();
        let module = llvm::LLVMModuleCreateWithName(b"karma\0".as_ptr() as *const _);
        let strings = Vec::new();

        Builder { context, module, strings }
    }


    /// Dump generated IR to stdout
    pub unsafe fn print_module(&self) {
        let c_str = llvm::LLVMPrintModuleToString(self.module);
        let cstring = CString::from_raw(c_str);
        let string = cstring.into_string().unwrap();
        println!("{}", string);
    }



    /// Create a new C compatible string from an ASCII string
    pub fn create_str_mut(&mut self, string: &str) -> *mut i8 {
        let cstring = CString::new(string).unwrap();
        let ptr = cstring.as_ptr();
        self.strings.push(cstring);
        ptr as *mut i8
    }
    
    /// Create a new C compatible string from an ASCII string
    pub fn create_str(&mut self, string: &str) -> *const i8 {
        self.create_str_mut(string) as *const i8
    }



    /// Get a function based on its name
    pub unsafe fn get_named_function(&mut self, name: &str) -> LLVMValueRef {
        llvm::LLVMGetNamedFunction(self.module, self.create_str(name))
    }

    /// Add a global variable with standard value
    pub unsafe fn add_global_variable(&mut self, name: &str, value: LLVMValueRef) -> LLVMValueRef {
        let kind = llvm::LLVMTypeOf(value);

        let variable = llvm::LLVMAddGlobal(self.module, kind, self.create_str(name));
        llvm::LLVMSetInitializer(variable, value);

        variable
    }

    /// Create a new function
    pub unsafe fn add_function(&mut self, name: &str, return_type: LLVMTypeRef, argument_types: &mut[LLVMTypeRef]) -> LLVMValueRef {
        let function_type = llvm::LLVMFunctionType(
            return_type, 
            argument_types.as_ptr() as *mut _,
            argument_types.len() as u32,
            0
        );

        llvm::LLVMAddFunction(
            self.module,
            self.create_str(name),
            function_type
        )
    }

    /// Create and append block to a function
    pub unsafe fn add_block(&mut self, function: LLVMValueRef, name: &str) -> LLVMBasicBlockRef {
        llvm::LLVMAppendBasicBlockInContext(
            self.context,
            function,
            self.create_str(name)
        )
    }

    /// Append instructions to a block
    pub unsafe fn build_block<F>(&mut self, block: LLVMBasicBlockRef, build: F) 
        where F: FnOnce(BlockBuilder)
    {
        let block_builder = BlockBuilder::new(self, block);

        build(block_builder)
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            llvm::LLVMDisposeModule(self.module);
            llvm::LLVMContextDispose(self.context);
        }
    }
}


pub struct BlockBuilder<'a> {
    parent: &'a mut Builder,
    builder: LLVMBuilderRef,
}


impl<'a> BlockBuilder<'a> {
    pub unsafe fn new(parent: &'a mut Builder, block: LLVMBasicBlockRef) -> BlockBuilder {
        let builder = llvm::LLVMCreateBuilderInContext(parent.context);
        llvm::LLVMPositionBuilderAtEnd(builder, block);

        BlockBuilder { parent, builder }
    }


    pub unsafe fn call_function(&mut self, 
                                function_name: &str, 
                                arguments: &mut [LLVMValueRef], 
                                name: &str) -> LLVMValueRef {
        let function = self.parent.get_named_function(function_name);
            
        let name = self.parent.create_str(name);
        llvm::LLVMBuildCall(self.builder, 
                            function, 
                            arguments.as_ptr() as *mut _, 
                            arguments.len() as u32, 
                            name
        )
    }

    pub unsafe fn return_value(&mut self,
                         value: LLVMValueRef) -> LLVMValueRef {
        llvm::LLVMBuildRet(self.builder, value)
    }
}

impl<'a> Drop for BlockBuilder<'a> {
    fn drop(&mut self) {
        unsafe {
            llvm::LLVMDisposeBuilder(self.builder);
        }
    }
}



pub unsafe fn void_type() -> LLVMTypeRef {
    llvm::LLVMVoidType()
}

pub unsafe fn i8_type() -> LLVMTypeRef {
    llvm::LLVMInt8Type()
}

pub unsafe fn i32_type() -> LLVMTypeRef {
    llvm::LLVMInt32Type()
}

pub unsafe fn i64_type() -> LLVMTypeRef {
    llvm::LLVMInt64Type()
}


pub unsafe fn i8_ptr_type() -> LLVMTypeRef {
    llvm::LLVMPointerType(i8_type(), 0)
}

pub unsafe fn i64_ptr_type() -> LLVMTypeRef {
    llvm::LLVMPointerType(i64_type(), 0)
}



pub unsafe fn i8_value(n: i8) -> LLVMValueRef {
    llvm::LLVMConstInt(i8_type(), n as u64, 1)
}

pub unsafe fn i32_value(n: i32) -> LLVMValueRef {
    llvm::LLVMConstInt(i32_type(), n as u64, 1)
}

pub unsafe fn i64_value(n: i64) -> LLVMValueRef {
    llvm::LLVMConstInt(i64_type(), n as u64, 1)
}

pub unsafe fn i64_ptr_value() -> LLVMValueRef {
    llvm::LLVMConstPointerNull(i64_ptr_type())
}
