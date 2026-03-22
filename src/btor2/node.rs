use std::fmt;
use std::num::NonZeroU32;

use super::sort::Sort;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(NonZeroU32);

impl NodeId {
    pub fn new(index: u32) -> Self {
        Self(NonZeroU32::new(index).expect("NodeId cannot be zero"))
    }

    pub fn index(self) -> usize {
        self.0.get() as usize
    }

    pub fn raw(self) -> u32 {
        self.0.get()
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "N{}", self.0)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Implies,
    Eq,
    Neq,
    Sgt,
    Ugt,
    Sgte,
    Ugte,
    Slt,
    Ult,
    Slte,
    Ulte,
    And,
    Or,
    Xor,
    Sll,
    Srl,
    Sra,
    Add,
    Sub,
    Mul,
    Sdiv,
    Udiv,
    Srem,
    Urem,
}

impl BinaryOp {
    pub fn btor2_name(self) -> &'static str {
        match self {
            Self::Implies => "implies",
            Self::Eq => "eq",
            Self::Neq => "neq",
            Self::Sgt => "sgt",
            Self::Ugt => "ugt",
            Self::Sgte => "sgte",
            Self::Ugte => "ugte",
            Self::Slt => "slt",
            Self::Ult => "ult",
            Self::Slte => "slte",
            Self::Ulte => "ulte",
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::Sll => "sll",
            Self::Srl => "srl",
            Self::Sra => "sra",
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Sdiv => "sdiv",
            Self::Udiv => "udiv",
            Self::Srem => "srem",
            Self::Urem => "urem",
        }
    }

    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            Self::Eq
                | Self::Neq
                | Self::Sgt
                | Self::Ugt
                | Self::Sgte
                | Self::Ugte
                | Self::Slt
                | Self::Ult
                | Self::Slte
                | Self::Ulte
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Not,
    Inc,
    Dec,
    Neg,
}

impl UnaryOp {
    pub fn btor2_name(&self) -> &'static str {
        match self {
            Self::Not => "not",
            Self::Inc => "inc",
            Self::Dec => "dec",
            Self::Neg => "neg",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Op {
    // Sorts
    Sort(Sort),
    // Constants
    Constd { sort: NodeId, value: u64 },
    Consth { sort: NodeId, value: u64 },
    Const { sort: NodeId, value: u64 },
    // Input / State
    Input { sort: NodeId, symbol: String },
    State { sort: NodeId, symbol: String },
    // Init / Next
    Init { sort: NodeId, state: NodeId, value: NodeId },
    Next { sort: NodeId, state: NodeId, value: NodeId },
    // Extension
    Sext { sort: NodeId, arg: NodeId, width: u32 },
    Uext { sort: NodeId, arg: NodeId, width: u32 },
    Slice { sort: NodeId, arg: NodeId, upper: u32, lower: u32 },
    // Unary
    Unary { kind: UnaryOp, sort: NodeId, arg: NodeId },
    // Binary
    Binary { kind: BinaryOp, sort: NodeId, left: NodeId, right: NodeId },
    // Concat
    Concat { sort: NodeId, left: NodeId, right: NodeId },
    // Array read
    Read { sort: NodeId, array: NodeId, index: NodeId },
    // Ternary
    Ite { sort: NodeId, cond: NodeId, then_val: NodeId, else_val: NodeId },
    Write { sort: NodeId, array: NodeId, index: NodeId, value: NodeId },
    // Properties
    Bad { cond: NodeId, symbol: String },
    Constraint { cond: NodeId, symbol: String },
}

pub struct Node {
    pub id: NodeId,
    pub op: Op,
    pub comment: Option<String>,
    pub nid: u32,
}
