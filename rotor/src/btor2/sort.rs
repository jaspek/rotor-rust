use super::node::NodeId;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Sort {
    Bitvec { width: u32 },
    Array { index: NodeId, element: NodeId },
}

impl Sort {
    pub fn bitvec_width(&self) -> Option<u32> {
        match self {
            Sort::Bitvec { width } => Some(*width),
            Sort::Array { .. } => None,
        }
    }

    pub fn is_bitvec(&self) -> bool {
        matches!(self, Sort::Bitvec { .. })
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Sort::Array { .. })
    }
}
