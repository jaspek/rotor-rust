use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::machine::sorts::{MachineConstants, MachineSorts};

/// Register file operations: reading and writing individual registers
/// via BTOR2 array read/write on the register state array.
pub struct RegisterFile;

impl RegisterFile {
    /// Read a register value from the register file state.
    /// Returns a machine-word-width node.
    pub fn load_register(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        reg_state: NodeId,
        reg_addr: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        builder.read(sorts.sid_machine_word, reg_state, reg_addr, comment)
    }

    /// Write a value to a register in the register file.
    /// Returns the updated register file state.
    /// Writing to x0 (zero register) is a no-op — the caller must handle this
    /// by conditionally selecting between the original and updated state.
    pub fn store_register(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        reg_state: NodeId,
        reg_addr: NodeId,
        value: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        builder.write(
            sorts.sid_register_state,
            reg_state,
            reg_addr,
            value,
            comment,
        )
    }

    /// Load the value of a specific register by index (0-31).
    pub fn load_register_by_index(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        reg_state: NodeId,
        reg_index: u32,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        let reg_addr = consts.nid_register(reg_index);
        Self::load_register(builder, sorts, reg_state, reg_addr, comment)
    }

    /// Store a value to a specific register by index, handling x0 as always-zero.
    /// Returns the (possibly unchanged) register file state.
    pub fn store_register_by_index(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        reg_state: NodeId,
        reg_index: u32,
        value: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        if reg_index == 0 {
            // Writing to x0 is a no-op
            reg_state
        } else {
            let reg_addr = consts.nid_register(reg_index);
            Self::store_register(builder, sorts, reg_state, reg_addr, value, comment)
        }
    }

    /// Conditionally store a value to a register identified by a runtime address node.
    /// Handles x0 by selecting the original state when rd == 0.
    pub fn conditional_store(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        reg_state: NodeId,
        rd_addr: NodeId,
        value: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        let updated = Self::store_register(builder, sorts, reg_state, rd_addr, value, comment);

        // If rd == 0, keep original state (x0 is hardwired to 0)
        let rd_is_zero = builder.eq_node(
            sorts.sid_boolean,
            rd_addr,
            consts.nid_register(0),
            Some("rd == x0?".to_string()),
        );
        builder.ite(
            sorts.sid_register_state,
            rd_is_zero,
            reg_state,
            updated,
            Some("x0 write guard".to_string()),
        )
    }
}
