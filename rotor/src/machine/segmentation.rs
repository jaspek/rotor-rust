use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::config::Config;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::riscv::elf_loader::LoadedBinary;

/// Page size used by the C reference for heap alignment.
pub const PAGE_SIZE: u64 = 4096;

fn align_up(value: u64, alignment: u64) -> u64 {
    (value + alignment - 1) & !(alignment - 1)
}

/// Segment boundary constants for one core.
///
/// Boundary semantics match the C reference (rotor.c):
///   code  = [code_start,  code_end)    from the ELF binary
///   data  = [data_start,  data_end)    from the ELF binary
///   heap  = [heap_start,  heap_end)    heap_start page-aligned AFTER data end
///   stack = [stack_start, stack_end)   stack_end = 2^vaddr_bits (top of the
///                                      virtual address space, e.g. 4 GB)
///
/// On targets where 2^vaddr_bits does not fit in a machine word (e.g. a
/// 32-bit machine with a 32-bit virtual address space) the stack end wraps
/// to zero, exactly like the C reference's `consth 4 00000000 ; end of stack
/// segment`. In that case the upper-bound comparison must be skipped
/// (`is_block_in_segment`, rotor.c:6519). `stack_end_wrapped` records this.
pub struct Segmentation {
    pub code_start: NodeId,
    pub code_end: NodeId,
    pub data_start: NodeId,
    pub data_end: NodeId,
    pub heap_start: NodeId,
    pub heap_end: NodeId,
    pub stack_start: NodeId,
    pub stack_end: NodeId,
    /// Highest valid virtual address constant, (1 << vaddr_bits) - 1.
    /// Used by `is_machine_word_virtual_address` checks (rotor.c:7552).
    pub highest_vaddr: NodeId,

    // Raw boundary values for use by other modules (SP/brk init, argv layout).
    pub code_start_val: u64,
    pub code_end_val: u64,
    pub data_start_val: u64,
    pub data_end_val: u64,
    pub heap_start_val: u64,
    pub heap_end_val: u64,
    pub stack_start_val: u64,
    /// 2^vaddr_bits — may exceed the machine word range (see stack_end_wrapped).
    pub stack_end_val: u64,
    pub stack_end_wrapped: bool,
    pub highest_vaddr_val: u64,
}

impl Segmentation {
    /// Create segment boundaries from a loaded binary and config.
    pub fn new(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        _consts: &MachineConstants,
        binary: &LoadedBinary,
        config: &Config,
    ) -> Self {
        let sid = sorts.sid_machine_word;
        let word_bits = config.machine_word_bits() as u64;
        let vaddr_bits = config.virtual_address_space as u64;

        let code_start_val = binary.code_start;
        let code_end_val = binary.code_start + binary.code_size;
        let data_start_val = binary.data_start;
        let data_end_val = binary.data_start + binary.data_size;

        // Heap starts at the next page boundary after the data segment
        // (C reference: data end 0x11008 -> heap start 0x12000).
        let heap_start_val = align_up(data_end_val, PAGE_SIZE);
        let heap_end_val = heap_start_val + config.heap_allowance;

        // Stack occupies the top of the virtual address space:
        // [2^vaddr - stack_allowance, 2^vaddr). For vaddr_bits = 32 this is
        // [0xFFFFF800, 0x100000000) with the default 2048-byte allowance,
        // matching the C reference exactly.
        let vaddr_top = 1u64 << vaddr_bits;
        let stack_start_val = vaddr_top - config.stack_allowance;
        let stack_end_val = vaddr_top;
        let highest_vaddr_val = vaddr_top - 1;

        // Does 2^vaddr_bits fit in a machine word? If not, the end constant
        // wraps (to 0 for vaddr_bits == word_bits) and upper-bound comparisons
        // must be skipped, as in C's is_block_in_segment.
        let stack_end_wrapped = vaddr_bits >= word_bits;
        let stack_end_repr = if stack_end_wrapped {
            // Truncated representation, matching C's printed constant.
            stack_end_val & ((1u128 << word_bits) as u64).wrapping_sub(1)
        } else {
            stack_end_val
        };

        let code_start = builder.constd(
            sid,
            code_start_val,
            Some(format!("start of code segment @ 0x{:x}", code_start_val)),
        );
        let code_end = builder.constd(
            sid,
            code_end_val,
            Some(format!("end of code segment @ 0x{:x}", code_end_val)),
        );
        let data_start = builder.constd(
            sid,
            data_start_val,
            Some(format!("start of data segment @ 0x{:x}", data_start_val)),
        );
        let data_end = builder.constd(
            sid,
            data_end_val,
            Some(format!("end of data segment @ 0x{:x}", data_end_val)),
        );
        let heap_start = builder.constd(
            sid,
            heap_start_val,
            Some(format!("start of heap segment @ 0x{:x}", heap_start_val)),
        );
        let heap_end = builder.constd(
            sid,
            heap_end_val,
            Some(format!(
                "static end of heap segment accommodating {} bytes",
                config.heap_allowance
            )),
        );
        let stack_start = builder.constd(
            sid,
            stack_start_val,
            Some(format!("static start of stack segment @ 0x{:x}", stack_start_val)),
        );
        let stack_end = builder.constd(
            sid,
            stack_end_repr,
            Some(format!(
                "end of stack segment accommodating {} bytes{}",
                config.stack_allowance,
                if stack_end_wrapped { " (wrapped)" } else { "" }
            )),
        );
        let highest_vaddr = builder.constd(
            sid,
            highest_vaddr_val,
            Some(format!("highest virtual address 0x{:x}", highest_vaddr_val)),
        );

        Self {
            code_start,
            code_end,
            data_start,
            data_end,
            heap_start,
            heap_end,
            stack_start,
            stack_end,
            highest_vaddr,
            code_start_val,
            code_end_val,
            data_start_val,
            data_end_val,
            heap_start_val,
            heap_end_val,
            stack_start_val,
            stack_end_val,
            stack_end_wrapped,
            highest_vaddr_val,
        }
    }

    /// `start <= vaddr < end`, skipping the upper bound when the segment end
    /// wrapped around the machine word (C's is_block_in_segment, rotor.c:6519).
    fn in_bounds(
        &self,
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        vaddr: NodeId,
        start: NodeId,
        end: NodeId,
        end_wrapped: bool,
        comment: &str,
    ) -> NodeId {
        let ge_start = builder.ugte(sorts.sid_boolean, vaddr, start, None);
        if end_wrapped {
            // Comparing with the end is unnecessary: it wrapped to zero, so
            // every address >= start is inside the segment (the address can't
            // exceed the word range by construction).
            ge_start
        } else {
            let lt_end = builder.ult(sorts.sid_boolean, vaddr, end, None);
            builder.and_node(sorts.sid_boolean, ge_start, lt_end, Some(comment.to_string()))
        }
    }

    /// Check if a virtual address falls within the code segment.
    pub fn is_in_code_segment(
        &self,
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        vaddr: NodeId,
    ) -> NodeId {
        self.in_bounds(
            builder,
            sorts,
            vaddr,
            self.code_start,
            self.code_end,
            false,
            "addr in code segment?",
        )
    }

    /// Check if a virtual address falls within the data segment.
    pub fn is_in_data_segment(
        &self,
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        vaddr: NodeId,
    ) -> NodeId {
        self.in_bounds(
            builder,
            sorts,
            vaddr,
            self.data_start,
            self.data_end,
            false,
            "addr in data segment?",
        )
    }

    /// Check if a virtual address falls within the heap segment.
    pub fn is_in_heap_segment(
        &self,
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        vaddr: NodeId,
    ) -> NodeId {
        self.in_bounds(
            builder,
            sorts,
            vaddr,
            self.heap_start,
            self.heap_end,
            false,
            "addr in heap segment?",
        )
    }

    /// Check if a virtual address falls within the stack segment.
    /// Wrap-aware: on targets where the stack end wraps to zero the upper
    /// bound comparison is skipped (exactly like the C reference).
    pub fn is_in_stack_segment(
        &self,
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        vaddr: NodeId,
    ) -> NodeId {
        self.in_bounds(
            builder,
            sorts,
            vaddr,
            self.stack_start,
            self.stack_end,
            self.stack_end_wrapped,
            "addr in stack segment?",
        )
    }

    /// vaddr fits in the virtual address space: vaddr <= highest_vaddr.
    /// (C's is_machine_word_virtual_address, rotor.c:7552.) Returns None when
    /// the virtual address space covers the whole machine word (the check is
    /// vacuous, C omits it entirely).
    pub fn is_machine_word_virtual_address(
        &self,
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        vaddr: NodeId,
    ) -> Option<NodeId> {
        if self.stack_end_wrapped {
            // vaddr space == word size: every word is a valid virtual address.
            None
        } else {
            Some(builder.ulte(
                sorts.sid_boolean,
                vaddr,
                self.highest_vaddr,
                Some("is machine word virtual address?".to_string()),
            ))
        }
    }

    /// Check if a virtual address is in any writable segment (data, heap, or stack).
    pub fn is_valid_write_address(
        &self,
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        vaddr: NodeId,
    ) -> NodeId {
        let in_data = self.is_in_data_segment(builder, sorts, vaddr);
        let in_heap = self.is_in_heap_segment(builder, sorts, vaddr);
        let in_stack = self.is_in_stack_segment(builder, sorts, vaddr);

        let data_or_heap = builder.or_node(sorts.sid_boolean, in_data, in_heap, None);
        builder.or_node(
            sorts.sid_boolean,
            data_or_heap,
            in_stack,
            Some("valid write address?".to_string()),
        )
    }

    /// Check if a virtual address is valid for reading (data, heap, stack, or code).
    pub fn is_valid_read_address(
        &self,
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        vaddr: NodeId,
    ) -> NodeId {
        let in_code = self.is_in_code_segment(builder, sorts, vaddr);
        let writable = self.is_valid_write_address(builder, sorts, vaddr);
        builder.or_node(
            sorts.sid_boolean,
            in_code,
            writable,
            Some("valid read address?".to_string()),
        )
    }

    /// Select the appropriate memory segment for a given address.
    /// Returns an ITE chain: if in data -> data_state, if in heap -> heap_state,
    /// if in stack -> stack_state, else -> data_state (default, will be caught by seg fault check).
    pub fn select_segment(
        &self,
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        vaddr: NodeId,
        data_state: NodeId,
        heap_state: NodeId,
        stack_state: NodeId,
        array_sid: NodeId,
    ) -> NodeId {
        let in_stack = self.is_in_stack_segment(builder, sorts, vaddr);
        let in_heap = self.is_in_heap_segment(builder, sorts, vaddr);

        let mut result = data_state; // default
        result = builder.ite(
            array_sid,
            in_heap,
            heap_state,
            result,
            Some("heap segment?".to_string()),
        );
        result = builder.ite(
            array_sid,
            in_stack,
            stack_state,
            result,
            Some("stack segment?".to_string()),
        );
        result
    }
}
