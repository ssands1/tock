//! Implementation of the architecture-specific portions of the kernel-userland
//! system call interface.

use core::fmt::Write;

pub struct SysCall();

impl SysCall {
    pub const unsafe fn new() -> SysCall {
        SysCall()
    }
}

impl kernel::syscall::UserspaceKernelBoundary for SysCall {
    type StoredState = ();

    unsafe fn initialize_process(
        &self,
        stack_pointer: *const usize,
        stack_size: usize,
        state: &mut Self::StoredState,
    ) -> Result<*const usize, ()> {
        Err(())
    }

    unsafe fn set_syscall_return_value(
        &self,
        stack_pointer: *const usize,
        state: &mut Self::StoredState,
        return_value: isize,
    ) {}

    unsafe fn switch_to_process(
        &self,
        stack_pointer: *const usize,
        _state: &mut Self::StoredState,
    ) -> (*mut usize, kernel::syscall::ContextSwitchReason) {
        (stack_pointer as *mut usize, kernel::syscall::ContextSwitchReason::Fault)
    }

    unsafe fn print_context(
        &self,
        stack_pointer: *const usize,
        state: &Self::StoredState,
        writer: &mut dyn Write,
    ) { }

    unsafe fn set_process_function(
        &self,
        stack_pointer: *const usize,
        remaining_stack_memory: usize,
        state: &mut Self::StoredState,
        callback: kernel::procs::FunctionCall,
    ) -> Result<*mut usize, *mut usize> {
        Err(stack_pointer as *mut usize)
    }

}
