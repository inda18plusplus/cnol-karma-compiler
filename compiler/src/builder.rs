
use llvm_sys::{
    self,
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
    pub fn new() -> Builder {
        unsafe {
            let context = llvm::LLVMContextCreate(); let module =
                llvm::LLVMModuleCreateWithName(b"karma\0".as_ptr() as *const _); let strings =
                Vec::new();

            Builder { context, module, strings }
        }
    }


    /// Dump generated IR to stdout
    pub fn print_module(&self) {
        let cstring = unsafe {
            let c_str = llvm::LLVMPrintModuleToString(self.module);
            CString::from_raw(c_str)
        };

        let string = cstring.into_string().unwrap();
        println!("{}", string);
    }


    pub fn is_working(&self) -> bool {
        use llvm_sys::analysis::*;
        unsafe {
            LLVMVerifyModule(
                self.module,
                LLVMVerifierFailureAction::LLVMPrintMessageAction,
                ptr::null_mut()
            ) == 0
        }
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
    pub fn get_named_function(&mut self, name: &str) -> LLVMValueRef {
        unsafe { llvm::LLVMGetNamedFunction(self.module, self.create_str(name)) }
    }

    /// Add a global variable with standard value
    pub fn add_global_variable(&mut self, name: &str, value: LLVMValueRef) -> LLVMValueRef {
        unsafe {
            let kind = llvm::LLVMTypeOf(value);

            let variable = llvm::LLVMAddGlobal(self.module, kind, self.create_str(name));
            llvm::LLVMSetInitializer(variable, value);

            variable
        }
    }

    /// Create a new function
    pub fn add_function(&mut self, 
                        name: &str, 
                        return_type: LLVMTypeRef, 
                        arguments: &[(&str, LLVMTypeRef)]) -> LLVMValueRef {
        unsafe { self.add_function_raw(name, return_type, arguments, false) } 
    }

    /// Create a new function with variadic arguments
    pub fn add_function_var_arg(&mut self, 
                                name: &str, 
                                return_type: LLVMTypeRef, 
                                arguments: &[(&str, LLVMTypeRef)]) -> LLVMValueRef {
        unsafe { self.add_function_raw(name, return_type, arguments, true) }
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
    pub fn get_param(&mut self,
                     function: LLVMValueRef,
                     index: u32) -> LLVMValueRef {
        unsafe { llvm::LLVMGetParam(function, index) }
    }


    /// Create and append block to a function
    pub fn add_block(&mut self, function: LLVMValueRef, name: &str) -> LLVMBasicBlockRef {
        unsafe {
            llvm::LLVMAppendBasicBlockInContext(
                self.context,
                function,
                self.create_str(name)
            )
        }
    }

    /// Append instructions to a block
    pub fn build_block<F>(&mut self, block: LLVMBasicBlockRef, build: F) 
        where F: FnOnce(BlockBuilder)
        {
            let block_builder = unsafe { BlockBuilder::new(self, block) };

            build(block_builder)
        }


    /// Add a constant string
    pub fn constant_string(&mut self,
                           string: &str) -> LLVMValueRef {
        let bytes: Vec<LLVMValueRef> = string.bytes().map(|b| i8_value(b as i8)).collect();
        unsafe { llvm::LLVMConstArray(i8_type(), bytes.as_ptr() as *mut _, bytes.len() as u32) }
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


pub enum Compare {
    Equal,
    Greater,
    GreaterOrEqual,
}


impl<'a> BlockBuilder<'a> {
    pub unsafe fn new(parent: &'a mut Builder, block: LLVMBasicBlockRef) -> BlockBuilder {
        let builder = llvm::LLVMCreateBuilderInContext(parent.context);
        llvm::LLVMPositionBuilderAtEnd(builder, block);

        BlockBuilder { parent, builder }
    }


    fn empty_str(&mut self) -> *const i8 {
        self.parent.create_str("")
    }


    pub fn call(&mut self, function: LLVMValueRef, arguments: &[LLVMValueRef]) -> LLVMValueRef {
        unsafe { 
            llvm::LLVMBuildCall(self.builder, 
                                function, 
                                arguments.as_ptr() as *mut _, 
                                arguments.len() as u32, 
                                self.empty_str()
            ) 
        }
    }

    pub fn call_function(&mut self, function_name: &str, arguments: &[LLVMValueRef]) -> LLVMValueRef {
        let function = self.parent.get_named_function(function_name);
        unsafe { 
            llvm::LLVMBuildCall(self.builder, 
                                function, 
                                arguments.as_ptr() as *mut _, 
                                arguments.len() as u32, 
                                self.empty_str()
            ) 
        }
    }


    pub fn return_void(&mut self) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildRetVoid(self.builder) }
    }

    pub fn return_value(&mut self, value: LLVMValueRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildRet(self.builder, value) }
    }


    pub fn branch(&mut self, target: LLVMBasicBlockRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildBr(self.builder, target) }
    }
    
    pub fn conditional_branch(&mut self, 
                              condition: LLVMValueRef,  
                              on_true: LLVMBasicBlockRef,
                              on_false: LLVMBasicBlockRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildCondBr(self.builder, condition, on_true, on_false) }
    }

    pub fn switch(&mut self,
                  condition: LLVMValueRef,
                  cases: &[(LLVMValueRef, LLVMBasicBlockRef)],
                  default: LLVMBasicBlockRef) {
        unsafe {
            let switch = llvm::LLVMBuildSwitch(self.builder, condition, default, cases.len() as u32);

            for (val, destination) in cases.iter() {
                llvm::LLVMAddCase(switch, *val, *destination);
            }
        }
    }

    pub fn load(&mut self, pointer_value: LLVMValueRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildLoad(self.builder, pointer_value, self.empty_str()) }
    }

    pub fn store(&mut self, value: LLVMValueRef, target: LLVMValueRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildStore(self.builder, value, target) }
    }


    pub fn get_element_offset(&mut self, array: LLVMValueRef, offset: LLVMValueRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildGEP(self.builder, array, [offset].as_mut_ptr(), 1, self.empty_str()) }
    }


    pub fn add(&mut self, lhs: LLVMValueRef, rhs: LLVMValueRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildAdd(self.builder, lhs, rhs, self.empty_str()) }
    }
    pub fn sub(&mut self, lhs: LLVMValueRef, rhs: LLVMValueRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildSub(self.builder, lhs, rhs, self.empty_str()) }
    }
    pub fn mul(&mut self, lhs: LLVMValueRef, rhs: LLVMValueRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildMul(self.builder, lhs, rhs, self.empty_str()) }
    }
    pub fn div(&mut self, lhs: LLVMValueRef, rhs: LLVMValueRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildSDiv(self.builder, lhs, rhs, self.empty_str()) }
    }
    pub fn modulo(&mut self, lhs: LLVMValueRef, rhs: LLVMValueRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildSRem(self.builder, lhs, rhs, self.empty_str()) }
    }

    pub fn bit_and(&mut self, lhs: LLVMValueRef, rhs: LLVMValueRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildAnd(self.builder, lhs, rhs, self.empty_str()) }
    }
    pub fn bit_or(&mut self, lhs: LLVMValueRef, rhs: LLVMValueRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildOr(self.builder, lhs, rhs, self.empty_str()) }
    }
    pub fn bit_xor(&mut self, lhs: LLVMValueRef, rhs: LLVMValueRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildXor(self.builder, lhs, rhs, self.empty_str()) }
    }
    pub fn bit_not(&mut self, val: LLVMValueRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildNot(self.builder, val, self.empty_str()) }
    }
    
    pub fn compare(&mut self, 
                   lhs: LLVMValueRef, 
                   comparison: Compare,
                   rhs: LLVMValueRef) -> LLVMValueRef {
        let op = match comparison {
            Compare::Equal => llvm_sys::LLVMIntPredicate::LLVMIntEQ,
            Compare::Greater => llvm_sys::LLVMIntPredicate::LLVMIntSGT,
            Compare::GreaterOrEqual => llvm_sys::LLVMIntPredicate::LLVMIntSGE,
        };

        unsafe { llvm::LLVMBuildICmp(self.builder, op, lhs, rhs, self.empty_str()) }
    }



    pub fn cast_int(&mut self, value: LLVMValueRef, target: LLVMTypeRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildIntCast(self.builder, value, target, self.empty_str()) }
    }

    pub fn zero_extend_int(&mut self, value: LLVMValueRef, target: LLVMTypeRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildZExt(self.builder, value, target, self.empty_str()) }
    }

    pub fn pointer_cast(&mut self,
                        value: LLVMValueRef,
                        target: LLVMTypeRef) -> LLVMValueRef {
        unsafe { llvm::LLVMBuildPointerCast(self.builder, value, target, self.empty_str()) }
    }
}

impl<'a> Drop for BlockBuilder<'a> {
    fn drop(&mut self) {
        unsafe {
            llvm::LLVMDisposeBuilder(self.builder);
        }
    }
}



pub fn void_type() -> LLVMTypeRef { unsafe { llvm::LLVMVoidType() } }

pub fn i1_type() -> LLVMTypeRef { unsafe { llvm::LLVMInt1Type() } }

pub fn i8_type() -> LLVMTypeRef { unsafe { llvm::LLVMInt8Type() } }

pub fn i32_type() -> LLVMTypeRef { unsafe { llvm::LLVMInt32Type() } }

pub fn i64_type() -> LLVMTypeRef { unsafe { llvm::LLVMInt64Type() } }


pub fn void_ptr_type() -> LLVMTypeRef { unsafe { llvm::LLVMPointerType(void_type(), 0) } }

pub fn i8_ptr_type() -> LLVMTypeRef { unsafe { llvm::LLVMPointerType(i8_type(), 0) } }

pub fn i64_ptr_type() -> LLVMTypeRef { unsafe { llvm::LLVMPointerType(i64_type(), 0) } }


pub fn i1_value(n: bool) -> LLVMValueRef { unsafe { llvm::LLVMConstInt(i1_type(), n as u64, 1) } }

pub fn i8_value(n: i8) -> LLVMValueRef { unsafe { llvm::LLVMConstInt(i8_type(), n as u64, 1) } }

pub fn i32_value(n: i32) -> LLVMValueRef { unsafe { llvm::LLVMConstInt(i32_type(), n as u64, 1) } }

pub fn i64_value(n: i64) -> LLVMValueRef { unsafe { llvm::LLVMConstInt(i64_type(), n as u64, 1) } }

pub fn i64_ptr_value() -> LLVMValueRef { unsafe { llvm::LLVMConstPointerNull(i64_ptr_type()) } }



