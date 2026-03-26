use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::config::Config;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::machine::segmentation::Segmentation;

/// Memory operations: loading and storing values at various widths
/// from/to byte-addressable segmented memory.
pub struct Memory;

impl Memory {
    // ========== LOAD OPERATIONS ==========

    /// Load a single byte from memory at address `vaddr`.
    pub fn load_byte(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        memory: NodeId,
        vaddr: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        builder.read(sorts.sid_byte, memory, vaddr, comment)
    }

    /// Load a half-word (16-bit) from memory, little-endian.
    pub fn load_half_word(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        memory: NodeId,
        vaddr: NodeId,
    ) -> NodeId {
        let byte0 = builder.read(sorts.sid_byte, memory, vaddr, Some("load byte 0".to_string()));
        let addr1 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_1, None);
        let byte1 = builder.read(sorts.sid_byte, memory, addr1, Some("load byte 1".to_string()));

        // Combine little-endian: byte1 | byte0
        builder.concat(sorts.sid_half_word, byte1, byte0, Some("half-word (LE)".to_string()))
    }

    /// Load a word (32-bit) from memory, little-endian.
    pub fn load_word(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        memory: NodeId,
        vaddr: NodeId,
    ) -> NodeId {
        let byte0 = builder.read(sorts.sid_byte, memory, vaddr, None);
        let addr1 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_1, None);
        let byte1 = builder.read(sorts.sid_byte, memory, addr1, None);
        let addr2 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_2, None);
        let byte2 = builder.read(sorts.sid_byte, memory, addr2, None);
        let addr3 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_3, None);
        let byte3 = builder.read(sorts.sid_byte, memory, addr3, None);

        // Little-endian: byte3 | byte2 | byte1 | byte0
        let hw_lo = builder.concat(sorts.sid_half_word, byte1, byte0, None);
        let hw_hi = builder.concat(sorts.sid_half_word, byte3, byte2, None);
        builder.concat(sorts.sid_single_word, hw_hi, hw_lo, Some("word (LE)".to_string()))
    }

    /// Load a double-word (64-bit) from memory, little-endian.
    pub fn load_double_word(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        memory: NodeId,
        vaddr: NodeId,
    ) -> NodeId {
        let byte0 = builder.read(sorts.sid_byte, memory, vaddr, None);
        let addr1 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_1, None);
        let byte1 = builder.read(sorts.sid_byte, memory, addr1, None);
        let addr2 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_2, None);
        let byte2 = builder.read(sorts.sid_byte, memory, addr2, None);
        let addr3 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_3, None);
        let byte3 = builder.read(sorts.sid_byte, memory, addr3, None);
        let addr4 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_4, None);
        let byte4 = builder.read(sorts.sid_byte, memory, addr4, None);
        let addr5 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_5, None);
        let byte5 = builder.read(sorts.sid_byte, memory, addr5, None);
        let addr6 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_6, None);
        let byte6 = builder.read(sorts.sid_byte, memory, addr6, None);
        let addr7 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_7, None);
        let byte7 = builder.read(sorts.sid_byte, memory, addr7, None);

        let hw0 = builder.concat(sorts.sid_half_word, byte1, byte0, None);
        let hw1 = builder.concat(sorts.sid_half_word, byte3, byte2, None);
        let hw2 = builder.concat(sorts.sid_half_word, byte5, byte4, None);
        let hw3 = builder.concat(sorts.sid_half_word, byte7, byte6, None);

        let w0 = builder.concat(sorts.sid_single_word, hw1, hw0, None);
        let w1 = builder.concat(sorts.sid_single_word, hw3, hw2, None);

        builder.concat(sorts.sid_double_word, w1, w0, Some("double-word (LE)".to_string()))
    }

    /// Load a value with sign extension based on the load instruction type.
    /// `load_width`: 1=byte, 2=half, 4=word, 8=double
    /// `sign_extend`: true for LB/LH/LW, false for LBU/LHU/LWU
    pub fn load_value(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        memory: NodeId,
        vaddr: NodeId,
        load_width: u32,
        sign_extend: bool,
        config: &Config,
    ) -> NodeId {
        let word_bits = config.machine_word_bits();

        match load_width {
            1 => {
                let byte = builder.read(sorts.sid_byte, memory, vaddr, Some("load byte".to_string()));
                let ext_bits = word_bits - 8;
                if sign_extend {
                    builder.sext(sorts.sid_machine_word, byte, ext_bits, Some("sign-extend byte".to_string()))
                } else {
                    builder.uext(sorts.sid_machine_word, byte, ext_bits, Some("zero-extend byte".to_string()))
                }
            }
            2 => {
                let hw = Self::load_half_word(builder, sorts, consts, memory, vaddr);
                let ext_bits = word_bits - 16;
                if sign_extend {
                    builder.sext(sorts.sid_machine_word, hw, ext_bits, Some("sign-extend half-word".to_string()))
                } else {
                    builder.uext(sorts.sid_machine_word, hw, ext_bits, Some("zero-extend half-word".to_string()))
                }
            }
            4 => {
                let w = Self::load_word(builder, sorts, consts, memory, vaddr);
                if word_bits > 32 {
                    let ext_bits = word_bits - 32;
                    if sign_extend {
                        builder.sext(sorts.sid_machine_word, w, ext_bits, Some("sign-extend word".to_string()))
                    } else {
                        builder.uext(sorts.sid_machine_word, w, ext_bits, Some("zero-extend word".to_string()))
                    }
                } else {
                    w
                }
            }
            8 => {
                Self::load_double_word(builder, sorts, consts, memory, vaddr)
            }
            _ => panic!("Invalid load width: {}", load_width),
        }
    }

    // ========== STORE OPERATIONS ==========

    /// Store a single byte to memory at address `vaddr`.
    pub fn store_byte(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        memory: NodeId,
        vaddr: NodeId,
        value: NodeId,
        array_sid: NodeId,
    ) -> NodeId {
        let byte_val = builder.slice(sorts.sid_byte, value, 7, 0, Some("extract byte".to_string()));
        builder.write(array_sid, memory, vaddr, byte_val, Some("store byte".to_string()))
    }

    /// Store a half-word (16-bit, little-endian) to memory.
    pub fn store_half_word(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        memory: NodeId,
        vaddr: NodeId,
        value: NodeId,
        array_sid: NodeId,
    ) -> NodeId {
        let byte0 = builder.slice(sorts.sid_byte, value, 7, 0, None);
        let byte1 = builder.slice(sorts.sid_byte, value, 15, 8, None);

        let mem1 = builder.write(array_sid, memory, vaddr, byte0, None);
        let addr1 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_1, None);
        builder.write(array_sid, mem1, addr1, byte1, Some("store half-word (LE)".to_string()))
    }

    /// Store a word (32-bit, little-endian) to memory.
    pub fn store_word(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        memory: NodeId,
        vaddr: NodeId,
        value: NodeId,
        array_sid: NodeId,
    ) -> NodeId {
        let byte0 = builder.slice(sorts.sid_byte, value, 7, 0, None);
        let byte1 = builder.slice(sorts.sid_byte, value, 15, 8, None);
        let byte2 = builder.slice(sorts.sid_byte, value, 23, 16, None);
        let byte3 = builder.slice(sorts.sid_byte, value, 31, 24, None);

        let mem1 = builder.write(array_sid, memory, vaddr, byte0, None);
        let addr1 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_1, None);
        let mem2 = builder.write(array_sid, mem1, addr1, byte1, None);
        let addr2 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_2, None);
        let mem3 = builder.write(array_sid, mem2, addr2, byte2, None);
        let addr3 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_3, None);
        builder.write(array_sid, mem3, addr3, byte3, Some("store word (LE)".to_string()))
    }

    /// Store a double-word (64-bit, little-endian) to memory.
    pub fn store_double_word(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        memory: NodeId,
        vaddr: NodeId,
        value: NodeId,
        array_sid: NodeId,
    ) -> NodeId {
        let byte0 = builder.slice(sorts.sid_byte, value, 7, 0, None);
        let byte1 = builder.slice(sorts.sid_byte, value, 15, 8, None);
        let byte2 = builder.slice(sorts.sid_byte, value, 23, 16, None);
        let byte3 = builder.slice(sorts.sid_byte, value, 31, 24, None);
        let byte4 = builder.slice(sorts.sid_byte, value, 39, 32, None);
        let byte5 = builder.slice(sorts.sid_byte, value, 47, 40, None);
        let byte6 = builder.slice(sorts.sid_byte, value, 55, 48, None);
        let byte7 = builder.slice(sorts.sid_byte, value, 63, 56, None);

        let mem1 = builder.write(array_sid, memory, vaddr, byte0, None);
        let a1 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_1, None);
        let mem2 = builder.write(array_sid, mem1, a1, byte1, None);
        let a2 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_2, None);
        let mem3 = builder.write(array_sid, mem2, a2, byte2, None);
        let a3 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_3, None);
        let mem4 = builder.write(array_sid, mem3, a3, byte3, None);
        let a4 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_4, None);
        let mem5 = builder.write(array_sid, mem4, a4, byte4, None);
        let a5 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_5, None);
        let mem6 = builder.write(array_sid, mem5, a5, byte5, None);
        let a6 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_6, None);
        let mem7 = builder.write(array_sid, mem6, a6, byte6, None);
        let a7 = builder.add(sorts.sid_machine_word, vaddr, consts.nid_machine_word_7, None);
        builder.write(array_sid, mem7, a7, byte7, Some("store double-word (LE)".to_string()))
    }

    /// Store a value of the given width to memory.
    pub fn store_value(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        memory: NodeId,
        vaddr: NodeId,
        value: NodeId,
        store_width: u32,
        array_sid: NodeId,
    ) -> NodeId {
        match store_width {
            1 => Self::store_byte(builder, sorts, memory, vaddr, value, array_sid),
            2 => Self::store_half_word(builder, sorts, consts, memory, vaddr, value, array_sid),
            4 => Self::store_word(builder, sorts, consts, memory, vaddr, value, array_sid),
            8 => Self::store_double_word(builder, sorts, consts, memory, vaddr, value, array_sid),
            _ => panic!("Invalid store width: {}", store_width),
        }
    }

    /// Check that all bytes in a multi-byte access are within the same segment.
    /// Returns a boolean node that is true if the access is properly contained.
    pub fn check_access_alignment(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        _consts: &MachineConstants,
        seg: &Segmentation,
        vaddr: NodeId,
        width: u32,
    ) -> NodeId {
        // Check that vaddr is valid and vaddr + width - 1 is also valid
        let valid_start = seg.is_valid_read_address(builder, sorts, vaddr);

        if width <= 1 {
            return valid_start;
        }

        let offset = builder.constd(
            sorts.sid_machine_word,
            (width - 1) as u64,
            None,
        );
        let last_byte_addr = builder.add(sorts.sid_machine_word, vaddr, offset, None);
        let valid_end = seg.is_valid_read_address(builder, sorts, last_byte_addr);

        builder.and_node(
            sorts.sid_boolean,
            valid_start,
            valid_end,
            Some(format!("{}-byte access bounds check", width)),
        )
    }
}
