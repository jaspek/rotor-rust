use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::riscv::elf_loader::LoadedBinary;

/// Segment boundary constants for one core.
pub struct Segmentation {
    pub code_start: NodeId,
    pub code_end: NodeId,
    pub data_start: NodeId,
    pub data_end: NodeId,
    pub heap_start: NodeId,
    pub heap_end: NodeId,
    pub stack_start: NodeId,
    pub stack_end: NodeId,
}

impl Segmentation {
    /// Create segment boundaries from a loaded binary and config.
    pub fn new(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        _consts: &MachineConstants,
        binary: &LoadedBinary,
        heap_allowance: u64,
        stack_allowance: u64,
    ) -> Self {
        let sid = sorts.sid_machine_word;

        let code_start = builder.constd(
            sid,
            binary.code_start,
            Some("code segment start".to_string()),
        );
        let code_end = builder.constd(
            sid,
            binary.code_start + binary.code_size,
            Some("code segment end".to_string()),
        );

        let data_start = builder.constd(
            sid,
            binary.data_start,
            Some("data segment start".to_string()),
        );
        let data_end = builder.constd(
            sid,
            binary.data_start + binary.data_size,
            Some("data segment end".to_string()),
        );

        // Heap starts right after data
        let heap_start_val = binary.data_start + binary.data_size;
        let heap_end_val = heap_start_val + heap_allowance;
        let heap_start =
            builder.constd(sid, heap_start_val, Some("heap segment start".to_string()));
        let heap_end = builder.constd(
            sid,
            heap_end_val,
            Some("heap segment end (max)".to_string()),
        );

        // Stack grows downward from top of virtual address space
        let vaddr_top = 1u64 << 31; // Use 2GB as default virtual address ceiling
        let stack_end_val = vaddr_top;
        let stack_start_val = stack_end_val - stack_allowance;
        let stack_start = builder.constd(
            sid,
            stack_start_val,
            Some("stack segment start".to_string()),
        );
        let stack_end = builder.constd(sid, stack_end_val, Some("stack segment end".to_string()));

        Self {
            code_start,
            code_end,
            data_start,
            data_end,
            heap_start,
            heap_end,
            stack_start,
            stack_end,
        }
    }

    /// Check if a virtual address falls within the data segment.
    pub fn is_in_data_segment(
        &self,
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        vaddr: NodeId,
    ) -> NodeId {
        let ge_start = builder.ugte(sorts.sid_boolean, vaddr, self.data_start, None);
        let lt_end = builder.ult(sorts.sid_boolean, vaddr, self.data_end, None);
        builder.and_node(
            sorts.sid_boolean,
            ge_start,
            lt_end,
            Some("addr in data segment?".to_string()),
        )
    }

    /// Check if a virtual address falls within the heap segment.
    pub fn is_in_heap_segment(
        &self,
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        vaddr: NodeId,
    ) -> NodeId {
        let ge_start = builder.ugte(sorts.sid_boolean, vaddr, self.heap_start, None);
        let lt_end = builder.ult(sorts.sid_boolean, vaddr, self.heap_end, None);
        builder.and_node(
            sorts.sid_boolean,
            ge_start,
            lt_end,
            Some("addr in heap segment?".to_string()),
        )
    }

    /// Check if a virtual address falls within the stack segment.
    pub fn is_in_stack_segment(
        &self,
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        vaddr: NodeId,
    ) -> NodeId {
        let ge_start = builder.ugte(sorts.sid_boolean, vaddr, self.stack_start, None);
        let lt_end = builder.ult(sorts.sid_boolean, vaddr, self.stack_end, None);
        builder.and_node(
            sorts.sid_boolean,
            ge_start,
            lt_end,
            Some("addr in stack segment?".to_string()),
        )
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
        let in_code = {
            let ge = builder.ugte(sorts.sid_boolean, vaddr, self.code_start, None);
            let lt = builder.ult(sorts.sid_boolean, vaddr, self.code_end, None);
            builder.and_node(sorts.sid_boolean, ge, lt, None)
        };
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
