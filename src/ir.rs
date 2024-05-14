use std::collections::HashMap;

use petgraph::graph::{DiGraph, NodeIndex};

#[derive(Debug, serde::Deserialize)]
pub struct Module {
    #[serde(rename = "FunctionList")]
    pub functions: Vec<Function>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Function {
    #[serde(rename = "GlobalIdentifier")]
    pub name: String,
    #[serde(rename = "Params")]
    pub params: Vec<Param>,
    #[serde(rename = "BasicBlock")]
    pub basic_blocks: Vec<BasicBlock>,
}

#[derive(Debug, Eq, Hash, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum Name {
    Name(String),
    Number(usize),
}

#[derive(Debug, serde::Deserialize)]
pub struct Type {
    #[serde(rename = "ID")]
    pub id: usize,
    #[serde(rename = "Name")]
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Param {
    #[serde(rename = "Name")]
    pub name: Name,
    #[serde(rename = "Type")]
    pub ty: Type,
}

#[derive(Debug, serde::Deserialize)]
pub struct BasicBlock {
    #[serde(rename = "Name")]
    pub name: Name,
    #[serde(rename = "Instructions")]
    pub insts: Vec<Instruction>,
    #[serde(rename = "Terminator")]
    pub term: Terminator,
}

#[derive(Debug, serde::Deserialize)]
pub struct Operand {
    #[serde(rename = "Constant")]
    pub constant: bool,
    #[serde(rename = "Name")]
    pub name: Option<Name>,
    #[serde(rename = "Type")]
    pub ty: Type,
}

#[derive(Debug, serde::Deserialize)]
pub struct Instruction {
    #[serde(rename = "Dest")]
    pub def: Option<Name>,
    #[serde(rename = "Uses")]
    pub uses: Vec<Operand>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Terminator {
    #[serde(rename = "Opcode")]
    pub opcode: usize,
    #[serde(rename = "Dest")]
    pub def: Option<Name>,
    #[serde(rename = "Uses")]
    pub uses: Vec<Operand>,
}

pub fn cfg(f: &Function) -> DiGraph<&Name, ()> {
    tracing::info!("cfg");
    let mut g = DiGraph::new();
    let blocks: HashMap<&'_ Name, (&'_ BasicBlock, NodeIndex)> = f
        .basic_blocks
        .iter()
        .map(|b| (&b.name, (b, g.add_node(&b.name))))
        .collect();
    for b in &f.basic_blocks {
        match (b.term.opcode, &b.term.uses[..]) {
            (2, &[ref l]) => {
                g.add_edge(blocks[&b.name].1, blocks[&l.name.as_ref().unwrap()].1, ());
            }
            (2, &[_, ref l, ref r]) => {
                g.add_edge(blocks[&b.name].1, blocks[&l.name.as_ref().unwrap()].1, ());
                g.add_edge(blocks[&b.name].1, blocks[&r.name.as_ref().unwrap()].1, ());
            }
            _ => tracing::warn!("not yet implemented: {:?}", b.term),
        }
    }
    g
}
