use std::io::{self, Write};

use super::builder::Btor2Builder;
use super::node::{Node, Op};
use super::sort::Sort;

pub struct Btor2Printer {
    print_comments: bool,
}

impl Btor2Printer {
    pub fn new(print_comments: bool) -> Self {
        Self { print_comments }
    }

    pub fn print(&self, builder: &Btor2Builder, out: &mut dyn Write) -> io::Result<()> {
        // Assign nids sequentially in arena order
        // (nodes are already in topological order by construction)
        let mut nid_map: Vec<u32> = vec![0; builder.node_count() + 1];
        let mut next_nid: u32 = 1;

        for node in builder.nodes() {
            nid_map[node.id.index()] = next_nid;
            next_nid += 1;
        }

        for node in builder.nodes() {
            let nid = nid_map[node.id.index()];
            self.print_node(node, nid, &nid_map, out)?;
        }

        Ok(())
    }

    fn nid_of(nid_map: &[u32], id: super::node::NodeId) -> u32 {
        nid_map[id.index()]
    }

    fn print_node(
        &self,
        node: &Node,
        nid: u32,
        nid_map: &[u32],
        out: &mut dyn Write,
    ) -> io::Result<()> {
        match &node.op {
            Op::Sort(sort) => {
                match sort {
                    Sort::Bitvec { width } => {
                        write!(out, "{} sort bitvec {}", nid, width)?;
                    }
                    Sort::Array { index, element } => {
                        write!(
                            out,
                            "{} sort array {} {}",
                            nid,
                            Self::nid_of(nid_map, *index),
                            Self::nid_of(nid_map, *element),
                        )?;
                    }
                }
            }
            Op::Constd { sort, value } => {
                write!(
                    out,
                    "{} constd {} {}",
                    nid,
                    Self::nid_of(nid_map, *sort),
                    *value as i64,
                )?;
            }
            Op::Consth { sort, value } => {
                write!(
                    out,
                    "{} consth {} {:x}",
                    nid,
                    Self::nid_of(nid_map, *sort),
                    value,
                )?;
            }
            Op::Const { sort, value } => {
                write!(
                    out,
                    "{} const {} {:b}",
                    nid,
                    Self::nid_of(nid_map, *sort),
                    value,
                )?;
            }
            Op::Input { sort, symbol } => {
                write!(
                    out,
                    "{} input {} {}",
                    nid,
                    Self::nid_of(nid_map, *sort),
                    symbol,
                )?;
            }
            Op::State { sort, symbol } => {
                if symbol.is_empty() {
                    write!(out, "{} state {}", nid, Self::nid_of(nid_map, *sort))?;
                } else {
                    write!(
                        out,
                        "{} state {} {}",
                        nid,
                        Self::nid_of(nid_map, *sort),
                        symbol,
                    )?;
                }
            }
            Op::Init { sort, state, value } => {
                write!(
                    out,
                    "{} init {} {} {}",
                    nid,
                    Self::nid_of(nid_map, *sort),
                    Self::nid_of(nid_map, *state),
                    Self::nid_of(nid_map, *value),
                )?;
            }
            Op::Next { sort, state, value } => {
                write!(
                    out,
                    "{} next {} {} {}",
                    nid,
                    Self::nid_of(nid_map, *sort),
                    Self::nid_of(nid_map, *state),
                    Self::nid_of(nid_map, *value),
                )?;
            }
            Op::Sext { sort, arg, width } => {
                write!(
                    out,
                    "{} sext {} {} {}",
                    nid,
                    Self::nid_of(nid_map, *sort),
                    Self::nid_of(nid_map, *arg),
                    width,
                )?;
            }
            Op::Uext { sort, arg, width } => {
                write!(
                    out,
                    "{} uext {} {} {}",
                    nid,
                    Self::nid_of(nid_map, *sort),
                    Self::nid_of(nid_map, *arg),
                    width,
                )?;
            }
            Op::Slice { sort, arg, upper, lower } => {
                write!(
                    out,
                    "{} slice {} {} {} {}",
                    nid,
                    Self::nid_of(nid_map, *sort),
                    Self::nid_of(nid_map, *arg),
                    upper,
                    lower,
                )?;
            }
            Op::Unary { kind, sort, arg } => {
                write!(
                    out,
                    "{} {} {} {}",
                    nid,
                    kind.btor2_name(),
                    Self::nid_of(nid_map, *sort),
                    Self::nid_of(nid_map, *arg),
                )?;
            }
            Op::Binary { kind, sort, left, right } => {
                write!(
                    out,
                    "{} {} {} {} {}",
                    nid,
                    kind.btor2_name(),
                    Self::nid_of(nid_map, *sort),
                    Self::nid_of(nid_map, *left),
                    Self::nid_of(nid_map, *right),
                )?;
            }
            Op::Concat { sort, left, right } => {
                write!(
                    out,
                    "{} concat {} {} {}",
                    nid,
                    Self::nid_of(nid_map, *sort),
                    Self::nid_of(nid_map, *left),
                    Self::nid_of(nid_map, *right),
                )?;
            }
            Op::Read { sort, array, index } => {
                write!(
                    out,
                    "{} read {} {} {}",
                    nid,
                    Self::nid_of(nid_map, *sort),
                    Self::nid_of(nid_map, *array),
                    Self::nid_of(nid_map, *index),
                )?;
            }
            Op::Ite { sort, cond, then_val, else_val } => {
                write!(
                    out,
                    "{} ite {} {} {} {}",
                    nid,
                    Self::nid_of(nid_map, *sort),
                    Self::nid_of(nid_map, *cond),
                    Self::nid_of(nid_map, *then_val),
                    Self::nid_of(nid_map, *else_val),
                )?;
            }
            Op::Write { sort, array, index, value } => {
                write!(
                    out,
                    "{} write {} {} {} {}",
                    nid,
                    Self::nid_of(nid_map, *sort),
                    Self::nid_of(nid_map, *array),
                    Self::nid_of(nid_map, *index),
                    Self::nid_of(nid_map, *value),
                )?;
            }
            Op::Bad { cond, symbol } => {
                if symbol.is_empty() {
                    write!(out, "{} bad {}", nid, Self::nid_of(nid_map, *cond))?;
                } else {
                    write!(
                        out,
                        "{} bad {} {}",
                        nid,
                        Self::nid_of(nid_map, *cond),
                        symbol,
                    )?;
                }
            }
            Op::Constraint { cond, symbol } => {
                if symbol.is_empty() {
                    write!(out, "{} constraint {}", nid, Self::nid_of(nid_map, *cond))?;
                } else {
                    write!(
                        out,
                        "{} constraint {} {}",
                        nid,
                        Self::nid_of(nid_map, *cond),
                        symbol,
                    )?;
                }
            }
        }

        if self.print_comments {
            if let Some(ref comment) = node.comment {
                write!(out, " ; {}", comment)?;
            }
        }

        writeln!(out)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_simple_model() {
        let mut b = Btor2Builder::new();
        let s1 = b.bitvec(1, Some("Boolean".to_string()));
        let s32 = b.bitvec(32, Some("32-bit".to_string()));
        let zero = b.constd(s32, 0, Some("zero".to_string()));
        let one = b.constd(s32, 1, Some("one".to_string()));
        let sum = b.add(s32, zero, one, Some("0 + 1".to_string()));
        let cond = b.eq_node(s1, sum, one, Some("sum == 1".to_string()));
        let _bad = b.bad(cond, "check", None);

        let mut out = Vec::new();
        let printer = Btor2Printer::new(true);
        printer.print(&b, &mut out).unwrap();
        let output = String::from_utf8(out).unwrap();

        assert!(output.contains("sort bitvec 1"));
        assert!(output.contains("sort bitvec 32"));
        assert!(output.contains("constd"));
        assert!(output.contains("add"));
        assert!(output.contains("eq"));
        assert!(output.contains("bad"));
    }
}
