// SPDX-License-Identifier: Apache-2.0

// Copied and modified for use without the `abi_x86_interrupt` feature.
// The `abi_x86_interrupt` feature is very unlikely to be stabilized.

// Copyright 2017 Philipp Oppermann. See the README.md
// file at the top-level directory of
// [this distribution](https://github.com/rust-osdev/x86_64).
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides types for the Interrupt Descriptor Table and its entries.

use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;

use bit_field::BitField;
use volatile::Volatile;
use x86_64::{PrivilegeLevel, VirtAddr};

/// An Interrupt Descriptor Table with 256 entries.
///
/// The first 32 entries are used for CPU exceptions. These entries can be either accessed through
/// fields on this struct or through an index operation, e.g. `idt[0]` returns the
/// first entry, the entry for the `divide_error` exception. Note that the index access is
/// not possible for entries for which an error code is pushed.
///
/// The remaining entries are used for interrupts. They can be accesed through index
/// operations on the idt, e.g. `idt[32]` returns the first interrupt entry, which is the 32th IDT
/// entry).
///
///
/// The field descriptions are taken from the
/// [AMD64 manual volume 2](https://support.amd.com/TechDocs/24593.pdf)
/// (with slight modifications).
#[allow(missing_debug_implementations)]
#[derive(Clone)]
#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    /// A divide error (`#DE`) occurs when the denominator of a DIV instruction or
    /// an IDIV instruction is 0. A `#DE` also occurs if the result is too large to be
    /// represented in the destination.
    ///
    /// The saved instruction pointer points to the instruction that caused the `#DE`.
    ///
    /// The vector number of the `#DE` exception is 0.
    pub divide_error: Entry<EnarxHandlerFunc>,

    /// When the debug-exception mechanism is enabled, a `#DB` exception can occur under any
    /// of the following circumstances:
    ///
    /// <details>
    ///
    /// - Instruction execution.
    /// - Instruction single stepping.
    /// - Data read.
    /// - Data write.
    /// - I/O read.
    /// - I/O write.
    /// - Task switch.
    /// - Debug-register access, or general detect fault (debug register access when DR7.GD=1).
    /// - Executing the INT1 instruction (opcode 0F1h).
    ///
    /// </details>
    ///
    /// `#DB` conditions are enabled and disabled using the debug-control register, `DR7`
    /// and `RFLAGS.TF`.
    ///
    /// In the following cases, the saved instruction pointer points to the instruction that
    /// caused the `#DB`:
    ///
    /// - Instruction execution.
    /// - Invalid debug-register access, or general detect.
    ///
    /// In all other cases, the instruction that caused the `#DB` is completed, and the saved
    /// instruction pointer points to the instruction after the one that caused the `#DB`.
    ///
    /// The vector number of the `#DB` exception is 1.
    pub debug: Entry<EnarxHandlerFunc>,

    /// An non maskable interrupt exception (NMI) occurs as a result of system logic
    /// signaling a non-maskable interrupt to the processor.
    ///
    /// The processor recognizes an NMI at an instruction boundary.
    /// The saved instruction pointer points to the instruction immediately following the
    /// boundary where the NMI was recognized.
    ///
    /// The vector number of the NMI exception is 2.
    pub non_maskable_interrupt: Entry<EnarxHandlerFunc>,

    /// A breakpoint (`#BP`) exception occurs when an `INT3` instruction is executed. The
    /// `INT3` is normally used by debug software to set instruction breakpoints by replacing
    ///
    /// The saved instruction pointer points to the byte after the `INT3` instruction.
    ///
    /// The vector number of the `#BP` exception is 3.
    pub breakpoint: Entry<EnarxHandlerFunc>,

    /// An overflow exception (`#OF`) occurs as a result of executing an `INTO` instruction
    /// while the overflow bit in `RFLAGS` is set to 1.
    ///
    /// The saved instruction pointer points to the instruction following the `INTO`
    /// instruction that caused the `#OF`.
    ///
    /// The vector number of the `#OF` exception is 4.
    pub overflow: Entry<EnarxHandlerFunc>,

    /// A bound-range exception (`#BR`) exception can occur as a result of executing
    /// the `BOUND` instruction. The `BOUND` instruction compares an array index (first
    /// operand) with the lower bounds and upper bounds of an array (second operand).
    /// If the array index is not within the array boundary, the `#BR` occurs.
    ///
    /// The saved instruction pointer points to the `BOUND` instruction that caused the `#BR`.
    ///
    /// The vector number of the `#BR` exception is 5.
    pub bound_range_exceeded: Entry<EnarxHandlerFunc>,

    /// An invalid opcode exception (`#UD`) occurs when an attempt is made to execute an
    /// invalid or undefined opcode. The validity of an opcode often depends on the
    /// processor operating mode.
    ///
    /// <details><summary>A `#UD` occurs under the following conditions:</summary>
    ///
    /// - Execution of any reserved or undefined opcode in any mode.
    /// - Execution of the `UD2` instruction.
    /// - Use of the `LOCK` prefix on an instruction that cannot be locked.
    /// - Use of the `LOCK` prefix on a lockable instruction with a non-memory target location.
    /// - Execution of an instruction with an invalid-operand type.
    /// - Execution of the `SYSENTER` or `SYSEXIT` instructions in long mode.
    /// - Execution of any of the following instructions in 64-bit mode: `AAA`, `AAD`,
    ///   `AAM`, `AAS`, `BOUND`, `CALL` (opcode 9A), `DAA`, `DAS`, `DEC`, `INC`, `INTO`,
    ///   `JMP` (opcode EA), `LDS`, `LES`, `POP` (`DS`, `ES`, `SS`), `POPA`, `PUSH` (`CS`,
    ///   `DS`, `ES`, `SS`), `PUSHA`, `SALC`.
    /// - Execution of the `ARPL`, `LAR`, `LLDT`, `LSL`, `LTR`, `SLDT`, `STR`, `VERR`, or
    ///   `VERW` instructions when protected mode is not enabled, or when virtual-8086 mode
    ///   is enabled.
    /// - Execution of any legacy SSE instruction when `CR4.OSFXSR` is cleared to 0.
    /// - Execution of any SSE instruction (uses `YMM`/`XMM` registers), or 64-bit media
    /// instruction (uses `MMXTM` registers) when `CR0.EM` = 1.
    /// - Execution of any SSE floating-point instruction (uses `YMM`/`XMM` registers) that
    /// causes a numeric exception when `CR4.OSXMMEXCPT` = 0.
    /// - Use of the `DR4` or `DR5` debug registers when `CR4.DE` = 1.
    /// - Execution of `RSM` when not in `SMM` mode.
    ///
    /// </details>
    ///
    /// The saved instruction pointer points to the instruction that caused the `#UD`.
    ///
    /// The vector number of the `#UD` exception is 6.
    pub invalid_opcode: Entry<EnarxHandlerFunc>,

    /// A device not available exception (`#NM`) occurs under any of the following conditions:
    ///
    /// <details>
    ///
    /// - An `FWAIT`/`WAIT` instruction is executed when `CR0.MP=1` and `CR0.TS=1`.
    /// - Any x87 instruction other than `FWAIT` is executed when `CR0.EM=1`.
    /// - Any x87 instruction is executed when `CR0.TS=1`. The `CR0.MP` bit controls whether the
    ///   `FWAIT`/`WAIT` instruction causes an `#NM` exception when `TS=1`.
    /// - Any 128-bit or 64-bit media instruction when `CR0.TS=1`.
    ///
    /// </details>
    ///
    /// The saved instruction pointer points to the instruction that caused the `#NM`.
    ///
    /// The vector number of the `#NM` exception is 7.
    pub device_not_available: Entry<EnarxHandlerFunc>,

    /// A double fault (`#DF`) exception can occur when a second exception occurs during
    /// the handling of a prior (first) exception or interrupt handler.
    ///
    /// <details>
    ///
    /// Usually, the first and second exceptions can be handled sequentially without
    /// resulting in a `#DF`. In this case, the first exception is considered _benign_, as
    /// it does not harm the ability of the processor to handle the second exception. In some
    /// cases, however, the first exception adversely affects the ability of the processor to
    /// handle the second exception. These exceptions contribute to the occurrence of a `#DF`,
    /// and are called _contributory exceptions_. The following exceptions are contributory:
    ///
    /// - Invalid-TSS Exception
    /// - Segment-Not-Present Exception
    /// - Stack Exception
    /// - General-Protection Exception
    ///
    /// A double-fault exception occurs in the following cases:
    ///
    /// - If a contributory exception is followed by another contributory exception.
    /// - If a divide-by-zero exception is followed by a contributory exception.
    /// - If a page  fault is followed by another page fault or a contributory exception.
    ///
    /// If a third interrupting event occurs while transferring control to the `#DF` handler,
    /// the processor shuts down.
    ///
    /// </details>
    ///
    /// The returned error code is always zero. The saved instruction pointer is undefined,
    /// and the program cannot be restarted.
    ///
    /// The vector number of the `#DF` exception is 8.
    pub double_fault: Entry<EnarxHandlerFunc>,

    /// This interrupt vector is reserved. It is for a discontinued exception originally used
    /// by processors that supported external x87-instruction coprocessors. On those processors,
    /// the exception condition is caused by an invalid-segment or invalid-page access on an
    /// x87-instruction coprocessor-instruction operand. On current processors, this condition
    /// causes a general-protection exception to occur.
    coprocessor_segment_overrun: Entry<EnarxHandlerFunc>,

    /// An invalid TSS exception (`#TS`) occurs only as a result of a control transfer through
    /// a gate descriptor that results in an invalid stack-segment reference using an `SS`
    /// selector in the TSS.
    ///
    /// The returned error code is the `SS` segment selector. The saved instruction pointer
    /// points to the control-transfer instruction that caused the `#TS`.
    ///
    /// The vector number of the `#TS` exception is 10.
    pub invalid_tss: Entry<EnarxHandlerFunc>,

    /// An segment-not-present exception (`#NP`) occurs when an attempt is made to load a
    /// segment or gate with a clear present bit.
    ///
    /// The returned error code is the segment-selector index of the segment descriptor
    /// causing the `#NP` exception. The saved instruction pointer points to the instruction
    /// that loaded the segment selector resulting in the `#NP`.
    ///
    /// The vector number of the `#NP` exception is 11.
    pub segment_not_present: Entry<EnarxHandlerFunc>,

    /// An stack segment exception (`#SS`) can occur in the following situations:
    ///
    /// - Implied stack references in which the stack address is not in canonical
    ///   form. Implied stack references include all push and pop instructions, and any
    ///   instruction using `RSP` or `RBP` as a base register.
    /// - Attempting to load a stack-segment selector that references a segment descriptor
    ///   containing a clear present bit.
    /// - Any stack access that fails the stack-limit check.
    ///
    /// The returned error code depends on the cause of the `#SS`. If the cause is a cleared
    /// present bit, the error code is the corresponding segment selector. Otherwise, the
    /// error code is zero. The saved instruction pointer points to the instruction that
    /// caused the `#SS`.
    ///
    /// The vector number of the `#NP` exception is 12.
    pub stack_segment_fault: Entry<EnarxHandlerFunc>,

    /// A general protection fault (`#GP`) can occur in various situations. Common causes include:
    ///
    /// - Executing a privileged instruction while `CPL > 0`.
    /// - Writing a 1 into any register field that is reserved, must be zero (MBZ).
    /// - Attempting to execute an SSE instruction specifying an unaligned memory operand.
    /// - Loading a non-canonical base address into the `GDTR` or `IDTR`.
    /// - Using WRMSR to write a read-only MSR.
    /// - Any long-mode consistency-check violation.
    ///
    /// The returned error code is a segment selector, if the cause of the `#GP` is
    /// segment-related, and zero otherwise. The saved instruction pointer points to
    /// the instruction that caused the `#GP`.
    ///
    /// The vector number of the `#GP` exception is 13.
    pub general_protection_fault: Entry<EnarxHandlerFunc>,

    /// A page fault (`#PF`) can occur during a memory access in any of the following situations:
    ///
    /// - A page-translation-table entry or physical page involved in translating the memory
    ///   access is not present in physical memory. This is indicated by a cleared present
    ///   bit in the translation-table entry.
    /// - An attempt is made by the processor to load the instruction TLB with a translation
    ///   for a non-executable page.
    /// - The memory access fails the paging-protection checks (user/supervisor, read/write,
    ///   or both).
    /// - A reserved bit in one of the page-translation-table entries is set to 1. A `#PF`
    ///   occurs for this reason only when `CR4.PSE=1` or `CR4.PAE=1`.
    ///
    /// The virtual (linear) address that caused the `#PF` is stored in the `CR2` register.
    /// The saved instruction pointer points to the instruction that caused the `#PF`.
    ///
    /// The page-fault error code is described by the
    /// [`PageFaultErrorCode`](struct.PageFaultErrorCode.html) struct.
    ///
    /// The vector number of the `#PF` exception is 14.
    pub page_fault: Entry<EnarxHandlerFunc>,

    /// vector nr. 15
    reserved_1: Entry<EnarxHandlerFunc>,

    /// The x87 Floating-Point Exception-Pending exception (`#MF`) is used to handle unmasked x87
    /// floating-point exceptions. In 64-bit mode, the x87 floating point unit is not used
    /// anymore, so this exception is only relevant when executing programs in the 32-bit
    /// compatibility mode.
    ///
    /// The vector number of the `#MF` exception is 16.
    pub x87_floating_point: Entry<EnarxHandlerFunc>,

    /// An alignment check exception (`#AC`) occurs when an unaligned-memory data reference
    /// is performed while alignment checking is enabled. An `#AC` can occur only when CPL=3.
    ///
    /// The returned error code is always zero. The saved instruction pointer points to the
    /// instruction that caused the `#AC`.
    ///
    /// The vector number of the `#AC` exception is 17.
    pub alignment_check: Entry<EnarxHandlerFunc>,

    /// The machine check exception (`#MC`) is model specific. Processor implementations
    /// are not required to support the `#MC` exception, and those implementations that do
    /// support `#MC` can vary in how the `#MC` exception mechanism works.
    ///
    /// There is no reliable way to restart the program.
    ///
    /// The vector number of the `#MC` exception is 18.
    pub machine_check: Entry<EnarxHandlerFunc>,

    /// The SIMD Floating-Point Exception (`#XF`) is used to handle unmasked SSE
    /// floating-point exceptions. The SSE floating-point exceptions reported by
    /// the `#XF` exception are (including mnemonics):
    ///
    /// - IE: Invalid-operation exception (also called #I).
    /// - DE: Denormalized-operand exception (also called #D).
    /// - ZE: Zero-divide exception (also called #Z).
    /// - OE: Overflow exception (also called #O).
    /// - UE: Underflow exception (also called #U).
    /// - PE: Precision exception (also called #P or inexact-result exception).
    ///
    /// The saved instruction pointer points to the instruction that caused the `#XF`.
    ///
    /// The vector number of the `#XF` exception is 19.
    pub simd_floating_point: Entry<EnarxHandlerFunc>,

    /// vector nr. 20
    pub virtualization: Entry<EnarxHandlerFunc>,

    /// vector nr. 21-29
    reserved_2: [Entry<EnarxHandlerFunc>; 8],

    /// The VMM Communication Exception (#VC) is always generated by hardware when an SEV-ES
    /// enabled guest is running and an NAE event occurs. The #VC exception is a precise, contributory, fault-
    /// type exception utilizing exception vector 29. This exception cannot be masked. The error code of the
    /// #VC exception is equal to the #VMEXIT code of the event that caused the NAE.
    /// In response to a #VC exception, a typical flow would involve the guest handler inspecting the error
    /// code to determine the cause of the exception and deciding what register state must be copied to the
    /// GHCB for the event to be handled. The handler should then execute the VMGEXIT instruction to
    /// create an AE and invoke the hypervisor. After a later VMRUN, guest execution will resume after the
    /// VMGEXIT instruction where the handler can view the results from the hypervisor and copy state from
    /// the GHCB back to its internal state as needed.
    /// Note that it is inadvisable for the hypervisor to set the VMCB intercept bit for the #VC exception as
    /// this would prevent proper handling of NAEs by the guest. Similarly, the hypervisor should avoid
    /// setting intercept bits for events that would occur in the #VC handler (such as IRET).
    ///
    /// The vector number of the ``#VC`` exception is 29.
    pub vmm_communication_exception: Entry<EnarxHandlerFunc>,

    /// The Security Exception (`#SX`) signals security-sensitive events that occur while
    /// executing the VMM, in the form of an exception so that the VMM may take appropriate
    /// action. (A VMM would typically intercept comparable sensitive events in the guest.)
    /// In the current implementation, the only use of the `#SX` is to redirect external INITs
    /// into an exception so that the VMM may — among other possibilities.
    ///
    /// The only error code currently defined is 1, and indicates redirection of INIT has occurred.
    ///
    /// The vector number of the ``#SX`` exception is 30.
    pub security_exception: Entry<EnarxHandlerFunc>,

    /// vector nr. 31
    reserved_3: Entry<EnarxHandlerFunc>,

    /// User-defined interrupts can be initiated either by system logic or software. They occur
    /// when:
    ///
    /// - System logic signals an external interrupt request to the processor. The signaling
    ///   mechanism and the method of communicating the interrupt vector to the processor are
    ///   implementation dependent.
    /// - Software executes an `INTn` instruction. The `INTn` instruction operand provides
    ///   the interrupt vector number.
    ///
    /// Both methods can be used to initiate an interrupt into vectors 0 through 255. However,
    /// because vectors 0 through 31 are defined or reserved by the AMD64 architecture,
    /// software should not use vectors in this range for purposes other than their defined use.
    ///
    /// The saved instruction pointer depends on the interrupt source:
    ///
    /// - External interrupts are recognized on instruction boundaries. The saved instruction
    ///   pointer points to the instruction immediately following the boundary where the
    ///   external interrupt was recognized.
    /// - If the interrupt occurs as a result of executing the INTn instruction, the saved
    ///   instruction pointer points to the instruction after the INTn.
    interrupts: [Entry<EnarxHandlerFunc>; 256 - 32],
}

impl InterruptDescriptorTable {
    /// Creates a new IDT filled with non-present entries.
    #[allow(clippy::new_without_default)]
    pub fn new() -> InterruptDescriptorTable {
        InterruptDescriptorTable {
            divide_error: Entry::missing(),
            debug: Entry::missing(),
            non_maskable_interrupt: Entry::missing(),
            breakpoint: Entry::missing(),
            overflow: Entry::missing(),
            bound_range_exceeded: Entry::missing(),
            invalid_opcode: Entry::missing(),
            device_not_available: Entry::missing(),
            double_fault: Entry::missing(),
            coprocessor_segment_overrun: Entry::missing(),
            invalid_tss: Entry::missing(),
            segment_not_present: Entry::missing(),
            stack_segment_fault: Entry::missing(),
            general_protection_fault: Entry::missing(),
            page_fault: Entry::missing(),
            reserved_1: Entry::missing(),
            x87_floating_point: Entry::missing(),
            alignment_check: Entry::missing(),
            machine_check: Entry::missing(),
            simd_floating_point: Entry::missing(),
            virtualization: Entry::missing(),
            reserved_2: [Entry::missing(); 8],
            vmm_communication_exception: Entry::missing(),
            security_exception: Entry::missing(),
            reserved_3: Entry::missing(),
            interrupts: [Entry::missing(); 256 - 32],
        }
    }

    /// Resets all entries of this IDT in place.
    pub fn reset(&mut self) {
        self.divide_error = Entry::missing();
        self.debug = Entry::missing();
        self.non_maskable_interrupt = Entry::missing();
        self.breakpoint = Entry::missing();
        self.overflow = Entry::missing();
        self.bound_range_exceeded = Entry::missing();
        self.invalid_opcode = Entry::missing();
        self.device_not_available = Entry::missing();
        self.double_fault = Entry::missing();
        self.coprocessor_segment_overrun = Entry::missing();
        self.invalid_tss = Entry::missing();
        self.segment_not_present = Entry::missing();
        self.stack_segment_fault = Entry::missing();
        self.general_protection_fault = Entry::missing();
        self.page_fault = Entry::missing();
        self.reserved_1 = Entry::missing();
        self.x87_floating_point = Entry::missing();
        self.alignment_check = Entry::missing();
        self.machine_check = Entry::missing();
        self.simd_floating_point = Entry::missing();
        self.virtualization = Entry::missing();
        self.reserved_2 = [Entry::missing(); 8];
        self.vmm_communication_exception = Entry::missing();
        self.security_exception = Entry::missing();
        self.reserved_3 = Entry::missing();
        self.interrupts = [Entry::missing(); 256 - 32];
    }

    /// Loads the IDT in the CPU using the `lidt` command.
    #[inline]
    pub fn load(&'static self) {
        unsafe { self.load_unsafe() }
    }

    /// Loads the IDT in the CPU using the `lidt` command.
    ///
    /// # Safety
    ///
    /// As long as it is the active IDT, you must ensure that:
    ///
    /// - `self` is never destroyed.
    /// - `self` always stays at the same memory location. It is recommended to wrap it in
    /// a `Box`.
    ///
    #[inline]
    pub unsafe fn load_unsafe(&self) {
        use x86_64::instructions::tables::lidt;
        lidt(&self.pointer());
    }

    /// Creates the descriptor pointer for this table. This pointer can only be
    /// safely used if the table is never modified or destroyed while in use.
    fn pointer(&self) -> x86_64::structures::DescriptorTablePointer {
        use core::mem::size_of;
        x86_64::structures::DescriptorTablePointer {
            base: VirtAddr::new(self as *const _ as u64),
            limit: size_of::<Self>().checked_sub(1).unwrap() as u16,
        }
    }
}

/// An Interrupt Descriptor Table entry.
///
/// The generic parameter can either be `HandlerFunc` or `HandlerFuncWithErrCode`, depending
/// on the interrupt vector.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Entry<F> {
    pointer_low: u16,
    gdt_selector: u16,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
    phantom: PhantomData<F>,
}

impl<T> fmt::Debug for Entry<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry")
            .field("handler_addr", &format_args!("{:#x}", self.handler_addr()))
            .field("gdt_selector", &self.gdt_selector)
            .field("options", &self.options)
            .finish()
    }
}

impl<F> Entry<F> {
    /// Creates a non-present IDT entry (but sets the must-be-one bits).
    #[inline]
    pub const fn missing() -> Self {
        Entry {
            gdt_selector: 0,
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::minimal(),
            reserved: 0,
            phantom: PhantomData,
        }
    }

    /// Set the handler address for the IDT entry and sets the present bit.
    ///
    /// For the code selector field, this function uses the code segment selector currently
    /// active in the CPU.
    ///
    /// The function returns a mutable reference to the entry's options that allows
    /// further customization.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `addr` is the address of a valid interrupt handler function,
    /// and the signature of such a function is correct for the entry type.
    #[inline]
    #[allow(clippy::integer_arithmetic)]
    pub unsafe fn set_handler_addr(&mut self, addr: VirtAddr) -> &mut EntryOptions {
        use x86_64::instructions::segmentation::{Segment, CS};

        let addr = addr.as_u64();

        self.pointer_low = addr as u16;
        self.pointer_middle = (addr >> 16) as u16;
        self.pointer_high = (addr >> 32) as u32;

        self.gdt_selector = CS::get_reg().0;

        self.options.set_present(true);
        &mut self.options
    }

    #[inline]
    #[allow(clippy::integer_arithmetic)]
    fn handler_addr(&self) -> u64 {
        self.pointer_low as u64
            | (self.pointer_middle as u64) << 16
            | (self.pointer_high as u64) << 32
    }
}

/// Our handler function is a asm naked function, because we don't want
/// to use the `abi_x86_interrupt` feature, which the `x86_64` crate relies on
/// for their idt implementation.
pub type EnarxHandlerFunc = unsafe extern "sysv64" fn() -> !;

impl Entry<EnarxHandlerFunc> {
    /// Set the handler function for the IDT entry and sets the present bit.
    ///
    /// For the code selector field, this function uses the code segment selector currently
    /// active in the CPU.
    ///
    /// The function returns a mutable reference to the entry's options that allows
    /// further customization.
    pub fn set_handler_fn(&mut self, handler: EnarxHandlerFunc) -> &mut EntryOptions {
        let handler = VirtAddr::new(handler as usize as u64);
        unsafe { self.set_handler_addr(handler) }
    }
}

/// Represents the options field of an IDT entry.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq)]
pub struct EntryOptions(u16);

impl fmt::Debug for EntryOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("EntryOptions")
            .field(&format_args!("{:#06x}", self.0))
            .finish()
    }
}

impl EntryOptions {
    /// Creates a minimal options field with all the must-be-one bits set.
    #[inline]
    const fn minimal() -> Self {
        EntryOptions(0b1110_0000_0000)
    }

    /// Set or reset the preset bit.
    #[inline]
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0.set_bit(15, present);
        self
    }

    /// Let the CPU disable hardware interrupts when the handler is invoked. By default,
    /// interrupts are disabled on handler invocation.
    #[inline]
    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.0.set_bit(8, !disable);
        self
    }

    /// Set the required privilege level (DPL) for invoking the handler. The DPL can be 0, 1, 2,
    /// or 3, the default is 0. If CPL < DPL, a general protection fault occurs.
    #[inline]
    pub fn set_privilege_level(&mut self, dpl: PrivilegeLevel) -> &mut Self {
        self.0.set_bits(13..15, dpl as u16);
        self
    }

    /// Assigns a Interrupt Stack Table (IST) stack to this handler. The CPU will then always
    /// switch to the specified stack before the handler is invoked. This allows kernels to
    /// recover from corrupt stack pointers (e.g., on kernel stack overflow).
    ///
    /// An IST stack is specified by an IST index between 0 and 6 (inclusive). Using the same
    /// stack for multiple interrupts can be dangerous when nested interrupts are possible.
    ///
    /// This function panics if the index is not in the range 0..7.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the caller must ensure that the passed stack index is
    /// valid and not used by other interrupts. Otherwise, memory safety violations are possible.
    #[inline]
    #[allow(clippy::integer_arithmetic)]
    pub unsafe fn set_stack_index(&mut self, index: u16) -> &mut Self {
        // The hardware IST index starts at 1, but our software IST index
        // starts at 0. Therefore we need to add 1 here.
        self.0.set_bits(0..3, index + 1);
        self
    }
}

// FIXME: The following can be removed, as soon as `x86_64` > 0.14.6 is released.
// https://github.com/rust-osdev/x86_64/pull/312

/// Wrapper type for the interrupt stack frame pushed by the CPU.
///
/// This type derefs to an [`InterruptStackFrameValue`], which allows reading the actual values.
///
/// This wrapper type ensures that no accidental modification of the interrupt stack frame
/// occurs, which can cause undefined behavior (see the [`as_mut`](InterruptStackFrame::as_mut)
/// method for more information).
#[repr(C)]
pub struct InterruptStackFrame {
    value: InterruptStackFrameValue,
}

impl InterruptStackFrame {
    /// Gives mutable access to the contents of the interrupt stack frame.
    ///
    /// The `Volatile` wrapper is used because LLVM optimizations remove non-volatile
    /// modifications of the interrupt stack frame.
    ///
    /// ## Safety
    ///
    /// This function is unsafe since modifying the content of the interrupt stack frame
    /// can easily lead to undefined behavior. For example, by writing an invalid value to
    /// the instruction pointer field, the CPU can jump to arbitrary code at the end of the
    /// interrupt.
    ///
    /// Also, it is not fully clear yet whether modifications of the interrupt stack frame are
    /// officially supported by LLVM's x86 interrupt calling convention.
    #[inline]
    pub unsafe fn as_mut(&mut self) -> Volatile<&mut InterruptStackFrameValue> {
        Volatile::new(&mut self.value)
    }
}

impl Deref for InterruptStackFrame {
    type Target = InterruptStackFrameValue;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl fmt::Debug for InterruptStackFrame {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

/// Represents the interrupt stack frame pushed by the CPU on interrupt or exception entry.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct InterruptStackFrameValue {
    /// This value points to the instruction that should be executed when the interrupt
    /// handler returns. For most interrupts, this value points to the instruction immediately
    /// following the last executed instruction. However, for some exceptions (e.g., page faults),
    /// this value points to the faulting instruction, so that the instruction is restarted on
    /// return. See the documentation of the [`InterruptDescriptorTable`] fields for more details.
    pub instruction_pointer: VirtAddr,
    /// The code segment selector, padded with zeros.
    pub code_segment: u64,
    /// The flags register before the interrupt handler was invoked.
    pub cpu_flags: u64,
    /// The stack pointer at the time of the interrupt.
    pub stack_pointer: VirtAddr,
    /// The stack segment descriptor at the time of the interrupt (often zero in 64-bit mode).
    pub stack_segment: u64,
}

impl fmt::Debug for InterruptStackFrameValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct Hex(u64);
        impl fmt::Debug for Hex {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{:#x}", self.0)
            }
        }

        let mut s = f.debug_struct("InterruptStackFrame");
        s.field("instruction_pointer", &self.instruction_pointer);
        s.field("code_segment", &self.code_segment);
        s.field("cpu_flags", &Hex(self.cpu_flags));
        s.field("stack_pointer", &self.stack_pointer);
        s.field("stack_segment", &self.stack_segment);
        s.finish()
    }
}