
use llvm_sys::{
    core as llvm,
    prelude::*
};

use std::{
    ffi::CString,
    ptr
};

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
    pub unsafe fn add_function(&mut self, 
                           name: &str, 
                           return_type: LLVMTypeRef, 
                           arguments: &[(&str, LLVMTypeRef)]) -> LLVMValueRef {
        self.add_function_raw(name, return_type, arguments, false)
    }

    /// Create a new function with variadic arguments
    pub unsafe fn add_function_var_arg(&mut self, 
                           name: &str, 
                           return_type: LLVMTypeRef, 
                           arguments: &[(&str, LLVMTypeRef)]) -> LLVMValueRef {
        self.add_function_raw(name, return_type, arguments, true)
    }

    /// Create a new function
    unsafe fn add_function_raw(&mut self, 
                               name: &str, 
                               return_type: LLVMTypeRef, 
                               arguments: &[(&str, LLVMTypeRef)],
                               is_var_arg: bool) -> LLVMValueRef {
        let argument_types: Vec<_> = arguments.iter().map(|arg| arg.1).collect();

        let function_type = llvm::LLVMFunctionType(
            return_type, 
            argument_types.as_ptr() as *mut _,
            argument_types.len() as u32,
            if is_var_arg {1} else {0}
        );

        let function = llvm::LLVMAddFunction(
            self.module,
            self.create_str(name),
            function_type
        );

        // name the parameters
        let params = vec![ptr::null::<LLVMValueRef>(); arguments.len()];
        llvm::LLVMGetParams(function, params.as_ptr() as *mut _);
        for (param, name) in params.into_iter().zip(arguments.iter().map(|arg| arg.0)) {
            let name = self.create_str(name);
            llvm::LLVMSetValueName(param as *mut _, name);
        }

        function
    }

    
    /// Get a parameter of a function
    pub unsafe fn get_param(&mut self,
                            function: LLVMValueRef,
                            index: u32) -> LLVMValueRef {
        llvm::LLVMGetParam(function, index)
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


    /// Add a constant string
    pub unsafe fn constant_string(&mut self,
                                  string: &str) -> LLVMValueRef {
        let bytes: Vec<LLVMValueRef> = string.bytes().map(|b| i8_value(b as i8)).collect();
        llvm::LLVMConstArray(i8_type(), bytes.as_ptr() as *mut _, bytes.len() as u32)
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
                                arguments: &[LLVMValueRef], 
                                output_name: &str) -> LLVMValueRef {
        let output = self.parent.create_str(output_name);

        let function = self.parent.get_named_function(function_name);
        llvm::LLVMBuildCall(self.builder, 
                            function, 
                            arguments.as_ptr() as *mut _, 
                            arguments.len() as u32, 
                            output
        )
    }


    pub unsafe fn return_void(&mut self) -> LLVMValueRef {
        llvm::LLVMBuildRetVoid(self.builder)
    }

    pub unsafe fn return_value(&mut self,
                         value: LLVMValueRef) -> LLVMValueRef {
        llvm::LLVMBuildRet(self.builder, value)
    }


    pub unsafe fn branch(&mut self,
                         target: LLVMBasicBlockRef) -> LLVMValueRef {
        llvm::LLVMBuildBr(self.builder, target)
    }

    pub unsafe fn switch(&mut self,
                         condition: LLVMValueRef,
                         cases: &[(LLVMValueRef, LLVMBasicBlockRef)],
                         default: LLVMBasicBlockRef) {
        let switch = llvm::LLVMBuildSwitch(self.builder, condition, default, cases.len() as u32);

        for (val, destination) in cases.iter() {
            llvm::LLVMAddCase(switch, *val, *destination);
        }
    }

    pub unsafe fn load(&mut self,
                           pointer_value: LLVMValueRef,
                           output_name: &str) -> LLVMValueRef {
        let output = self.parent.create_str(output_name);

        llvm::LLVMBuildLoad(self.builder, pointer_value, output)
    }

    pub unsafe fn store(&mut self,
                            value: LLVMValueRef,
                            target: LLVMValueRef) -> LLVMValueRef {
        llvm::LLVMBuildStore(self.builder, value, target)
    }


    pub unsafe fn get_element_offset(&mut self,
                                     array: LLVMValueRef,
                                     offset: LLVMValueRef,
                                     output_name: &str) -> LLVMValueRef {
        let output = self.parent.create_str(output_name);

        llvm::LLVMBuildGEP(self.builder, array, [offset].as_mut_ptr(), 1, output)
    }

    pub unsafe fn add(&mut self,
                      lhs: LLVMValueRef,
                      rhs: LLVMValueRef,
                      output_name: &str) -> LLVMValueRef {
        let output = self.parent.create_str(output_name);

        llvm::LLVMBuildAdd(self.builder, lhs, rhs, output)
    }


    pub unsafe fn cast_int(&mut self,
                           value: LLVMValueRef,
                           target: LLVMTypeRef,
                           output_name: &str) -> LLVMValueRef {
        let output = self.parent.create_str(output_name);

        llvm::LLVMBuildIntCast(self.builder, value, target, output)
    }

    pub unsafe fn pointer_cast(&mut self,
                               value: LLVMValueRef,
                               target: LLVMTypeRef,
                               output_name: &str) -> LLVMValueRef {
        let output = self.parent.create_str(output_name);

        llvm::LLVMBuildPointerCast(self.builder, value, target, output)
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
