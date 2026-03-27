use std::collections::HashMap;

use super::node::{BinaryOp, Node, NodeId, Op, UnaryOp};
use super::sort::Sort;

pub struct Btor2Builder {
    nodes: Vec<Node>,
    dedup: HashMap<Op, NodeId>,
    next_index: u32,
    enable_cse: bool,
}

impl Default for Btor2Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Btor2Builder {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            dedup: HashMap::new(),
            next_index: 1,
            enable_cse: true,
        }
    }

    fn alloc_id(&mut self) -> NodeId {
        let id = NodeId::new(self.next_index);
        self.next_index += 1;
        id
    }

    fn intern(&mut self, op: Op, comment: Option<String>) -> NodeId {
        #[allow(clippy::collapsible_if)]
        if self.enable_cse {
            if let Some(&existing) = self.dedup.get(&op) {
                return existing;
            }
        }
        let id = self.alloc_id();
        let node = Node {
            id,
            op: op.clone(),
            comment,
            nid: 0,
        };
        self.nodes.push(node);
        if self.enable_cse {
            self.dedup.insert(op, id);
        }
        id
    }

    pub fn set_cse(&mut self, enabled: bool) {
        self.enable_cse = enabled;
    }

    // --- Sort constructors ---

    pub fn bitvec(&mut self, width: u32, comment: impl Into<Option<String>>) -> NodeId {
        self.intern(Op::Sort(Sort::Bitvec { width }), comment.into())
    }

    pub fn array(
        &mut self,
        index_sort: NodeId,
        element_sort: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.intern(
            Op::Sort(Sort::Array {
                index: index_sort,
                element: element_sort,
            }),
            comment.into(),
        )
    }

    // --- Constant constructors ---

    pub fn constd(
        &mut self,
        sort: NodeId,
        value: u64,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.intern(Op::Constd { sort, value }, comment.into())
    }

    pub fn consth(
        &mut self,
        sort: NodeId,
        value: u64,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.intern(Op::Consth { sort, value }, comment.into())
    }

    pub fn const_bin(
        &mut self,
        sort: NodeId,
        value: u64,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.intern(Op::Const { sort, value }, comment.into())
    }

    // --- Input / State ---

    pub fn input(
        &mut self,
        sort: NodeId,
        symbol: &str,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.intern(
            Op::Input {
                sort,
                symbol: symbol.to_string(),
            },
            comment.into(),
        )
    }

    pub fn state(
        &mut self,
        sort: NodeId,
        symbol: &str,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        // States are never deduplicated — each is unique
        let was_cse = self.enable_cse;
        self.enable_cse = false;
        let id = self.intern(
            Op::State {
                sort,
                symbol: symbol.to_string(),
            },
            comment.into(),
        );
        self.enable_cse = was_cse;
        id
    }

    // --- Init / Next ---

    pub fn init(
        &mut self,
        sort: NodeId,
        state: NodeId,
        value: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        // Init lines are never deduplicated
        let was_cse = self.enable_cse;
        self.enable_cse = false;
        let id = self.intern(Op::Init { sort, state, value }, comment.into());
        self.enable_cse = was_cse;
        id
    }

    pub fn next(
        &mut self,
        sort: NodeId,
        state: NodeId,
        value: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        // Next lines are never deduplicated
        let was_cse = self.enable_cse;
        self.enable_cse = false;
        let id = self.intern(Op::Next { sort, state, value }, comment.into());
        self.enable_cse = was_cse;
        id
    }

    // --- Extension / Slice ---

    pub fn sext(
        &mut self,
        sort: NodeId,
        arg: NodeId,
        width: u32,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.intern(Op::Sext { sort, arg, width }, comment.into())
    }

    pub fn uext(
        &mut self,
        sort: NodeId,
        arg: NodeId,
        width: u32,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.intern(Op::Uext { sort, arg, width }, comment.into())
    }

    pub fn slice(
        &mut self,
        sort: NodeId,
        arg: NodeId,
        upper: u32,
        lower: u32,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.intern(
            Op::Slice {
                sort,
                arg,
                upper,
                lower,
            },
            comment.into(),
        )
    }

    // --- Unary ---

    pub fn not(&mut self, sort: NodeId, arg: NodeId, comment: impl Into<Option<String>>) -> NodeId {
        self.intern(
            Op::Unary {
                kind: UnaryOp::Not,
                sort,
                arg,
            },
            comment.into(),
        )
    }

    pub fn inc(&mut self, sort: NodeId, arg: NodeId, comment: impl Into<Option<String>>) -> NodeId {
        self.intern(
            Op::Unary {
                kind: UnaryOp::Inc,
                sort,
                arg,
            },
            comment.into(),
        )
    }

    pub fn dec(&mut self, sort: NodeId, arg: NodeId, comment: impl Into<Option<String>>) -> NodeId {
        self.intern(
            Op::Unary {
                kind: UnaryOp::Dec,
                sort,
                arg,
            },
            comment.into(),
        )
    }

    pub fn neg(&mut self, sort: NodeId, arg: NodeId, comment: impl Into<Option<String>>) -> NodeId {
        self.intern(
            Op::Unary {
                kind: UnaryOp::Neg,
                sort,
                arg,
            },
            comment.into(),
        )
    }

    // --- Binary ---

    pub fn binary(
        &mut self,
        kind: BinaryOp,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.intern(
            Op::Binary {
                kind,
                sort,
                left,
                right,
            },
            comment.into(),
        )
    }

    // Convenience methods for common binary ops
    pub fn eq_node(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Eq, sort, left, right, comment)
    }

    pub fn neq(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Neq, sort, left, right, comment)
    }

    pub fn add(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Add, sort, left, right, comment)
    }

    pub fn sub(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Sub, sort, left, right, comment)
    }

    pub fn mul(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Mul, sort, left, right, comment)
    }

    pub fn and_node(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::And, sort, left, right, comment)
    }

    pub fn or_node(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Or, sort, left, right, comment)
    }

    pub fn xor_node(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Xor, sort, left, right, comment)
    }

    pub fn sll(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Sll, sort, left, right, comment)
    }

    pub fn srl(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Srl, sort, left, right, comment)
    }

    pub fn sra(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Sra, sort, left, right, comment)
    }

    pub fn ult(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Ult, sort, left, right, comment)
    }

    pub fn ulte(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Ulte, sort, left, right, comment)
    }

    pub fn ugt(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Ugt, sort, left, right, comment)
    }

    pub fn ugte(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Ugte, sort, left, right, comment)
    }

    pub fn slt(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Slt, sort, left, right, comment)
    }

    pub fn slte(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Slte, sort, left, right, comment)
    }

    pub fn sgt(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Sgt, sort, left, right, comment)
    }

    pub fn sgte(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Sgte, sort, left, right, comment)
    }

    pub fn implies(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Implies, sort, left, right, comment)
    }

    pub fn udiv(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Udiv, sort, left, right, comment)
    }

    pub fn sdiv(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Sdiv, sort, left, right, comment)
    }

    pub fn urem(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Urem, sort, left, right, comment)
    }

    pub fn srem(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.binary(BinaryOp::Srem, sort, left, right, comment)
    }

    // --- Concat ---

    pub fn concat(
        &mut self,
        sort: NodeId,
        left: NodeId,
        right: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.intern(Op::Concat { sort, left, right }, comment.into())
    }

    // --- Array read ---

    pub fn read(
        &mut self,
        sort: NodeId,
        array: NodeId,
        index: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.intern(Op::Read { sort, array, index }, comment.into())
    }

    // --- Ternary ---

    pub fn ite(
        &mut self,
        sort: NodeId,
        cond: NodeId,
        then_val: NodeId,
        else_val: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.intern(
            Op::Ite {
                sort,
                cond,
                then_val,
                else_val,
            },
            comment.into(),
        )
    }

    pub fn write(
        &mut self,
        sort: NodeId,
        array: NodeId,
        index: NodeId,
        value: NodeId,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        self.intern(
            Op::Write {
                sort,
                array,
                index,
                value,
            },
            comment.into(),
        )
    }

    // --- Properties ---

    pub fn bad(
        &mut self,
        cond: NodeId,
        symbol: &str,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        let was_cse = self.enable_cse;
        self.enable_cse = false;
        let id = self.intern(
            Op::Bad {
                cond,
                symbol: symbol.to_string(),
            },
            comment.into(),
        );
        self.enable_cse = was_cse;
        id
    }

    pub fn constraint(
        &mut self,
        cond: NodeId,
        symbol: &str,
        comment: impl Into<Option<String>>,
    ) -> NodeId {
        let was_cse = self.enable_cse;
        self.enable_cse = false;
        let id = self.intern(
            Op::Constraint {
                cond,
                symbol: symbol.to_string(),
            },
            comment.into(),
        );
        self.enable_cse = was_cse;
        id
    }

    // --- Accessors ---

    pub fn get(&self, id: NodeId) -> &Node {
        // Nodes are stored starting at index 0, but IDs start at 1
        &self.nodes[id.index() - 1]
    }

    pub fn get_op(&self, id: NodeId) -> &Op {
        &self.get(id).op
    }

    pub fn get_sort_of(&self, id: NodeId) -> Option<NodeId> {
        match &self.get(id).op {
            Op::Sort(_) => None,
            Op::Constd { sort, .. }
            | Op::Consth { sort, .. }
            | Op::Const { sort, .. }
            | Op::Input { sort, .. }
            | Op::State { sort, .. }
            | Op::Init { sort, .. }
            | Op::Next { sort, .. }
            | Op::Sext { sort, .. }
            | Op::Uext { sort, .. }
            | Op::Slice { sort, .. }
            | Op::Unary { sort, .. }
            | Op::Binary { sort, .. }
            | Op::Concat { sort, .. }
            | Op::Read { sort, .. }
            | Op::Ite { sort, .. }
            | Op::Write { sort, .. } => Some(*sort),
            Op::Bad { .. } | Op::Constraint { .. } => None,
        }
    }

    pub fn get_bitvec_width(&self, sort_id: NodeId) -> Option<u32> {
        match &self.get(sort_id).op {
            Op::Sort(Sort::Bitvec { width }) => Some(*width),
            _ => None,
        }
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get a node by its arena index (1-based, matching NodeId::index()).
    pub fn node_by_index(&self, index: usize) -> &Node {
        &self.nodes[index - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_sorts() {
        let mut b = Btor2Builder::new();
        let s1 = b.bitvec(1, None);
        let s8 = b.bitvec(8, None);
        let s32 = b.bitvec(32, None);

        assert_ne!(s1, s8);
        assert_ne!(s8, s32);

        // CSE: same sort returns same id
        let s1_dup = b.bitvec(1, None);
        assert_eq!(s1, s1_dup);
    }

    #[test]
    fn test_constant_cse() {
        let mut b = Btor2Builder::new();
        let s32 = b.bitvec(32, None);
        let c0a = b.constd(s32, 0, None);
        let c0b = b.constd(s32, 0, None);
        assert_eq!(c0a, c0b);

        let c1 = b.constd(s32, 1, None);
        assert_ne!(c0a, c1);
    }

    #[test]
    fn test_binary_cse() {
        let mut b = Btor2Builder::new();
        let s32 = b.bitvec(32, None);
        let c0 = b.constd(s32, 0, None);
        let c1 = b.constd(s32, 1, None);

        let add1 = b.add(s32, c0, c1, None);
        let add2 = b.add(s32, c0, c1, None);
        assert_eq!(add1, add2);
    }

    #[test]
    fn test_state_no_cse() {
        let mut b = Btor2Builder::new();
        let s32 = b.bitvec(32, None);
        let st1 = b.state(s32, "pc", None);
        let st2 = b.state(s32, "pc", None);
        // States should NOT be deduplicated
        assert_ne!(st1, st2);
    }

    #[test]
    fn test_array_sort() {
        let mut b = Btor2Builder::new();
        let s5 = b.bitvec(5, None);
        let s64 = b.bitvec(64, None);
        let arr = b.array(s5, s64, Some("register file".to_string()));

        match &b.get(arr).op {
            Op::Sort(Sort::Array { index, element }) => {
                assert_eq!(*index, s5);
                assert_eq!(*element, s64);
            }
            _ => panic!("Expected array sort"),
        }
    }
}
