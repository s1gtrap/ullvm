use std::collections::{HashMap, HashSet};
use std::fmt;

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

#[derive(Eq, Hash, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum Name {
    Name(String),
    Number(usize),
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Name::Name(n) => write!(f, "%{}", n),
            Name::Number(n) => write!(f, "%{}", n),
        }
    }
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

pub fn lva(f: &Function) -> Vec<(HashSet<&Name>, HashSet<&Name>)> {
    tracing::info!("lva");
    let (_, block_indices) = f
        .basic_blocks
        .iter()
        .fold((0, HashMap::new()), |(l, mut m), b| {
            m.insert(l, b);
            (l + b.insts.len() + 1, m)
        });
    tracing::info!("{block_indices:#?}");
    let mut lives = vec![
        (HashSet::new(), HashSet::new());
        f.basic_blocks.iter().map(|b| b.insts.len() + 1).sum()
    ];
    for (i, (ref mut r#in, ref mut out)) in lives.iter_mut().enumerate().rev() {
        tracing::info!("i = {i}");
        let (block_idx, block) = block_indices
            .iter()
            .filter(|&(j, _)| *j <= i)
            .max_by_key(|&(j, _)| *j)
            .unwrap();
        tracing::info!("{block_idx}");
        if let Some(inst) = &block.insts.get(i - block_idx) {
            tracing::info!(" insts[{}] = {:?}", i - block_idx, inst);
            let def = &block.insts[i - block_idx].def;
            let def: HashSet<_> = def.iter().collect();
            let r#use: HashSet<_> = block.insts[i - block_idx]
                .uses
                .iter()
                .filter_map(|o| {
                    if !o.constant {
                        Some(o.name.as_ref().unwrap())
                    } else {
                        None
                    }
                })
                .collect();
            tracing::info!("use[{i}] = {:?}", r#use);
            *r#in = r#use.union(&(&*out - &def)).cloned().collect();
            tracing::info!(" in[{i}] = {:?}", r#use);
        } else {
            tracing::info!(" term = {:?}", &block.term);
            let def = &block.term.def;
            let def: HashSet<_> = def.iter().collect();
            let r#use: HashSet<_> = block
                .term
                .uses
                .iter()
                .filter_map(|o| {
                    if !o.constant {
                        Some(o.name.as_ref().unwrap())
                    } else {
                        None
                    }
                })
                .collect();
            tracing::info!("use[{i}] = {:?}", r#use);
            *r#in = r#use.union(&(&*out - &def)).cloned().collect();
            tracing::info!(" in[{i}] = {:?}", r#use);
        }
    }
    lives
}

#[test]
fn test_lva() {
    tracing_subscriber::fmt::init();
    assert_eq!(
        lva(&Function {
            name: "main".to_string(),
            params: vec![
                Param {
                    name: Name::Name("argc".to_string()),
                    ty: Type {
                        id: 13,
                        name: "i32".to_string(),
                    },
                },
                Param {
                    name: Name::Name("argv".to_string()),
                    ty: Type {
                        id: 15,
                        name: "ptr".to_string(),
                    },
                },
            ],
            basic_blocks: vec![BasicBlock {
                name: Name::Number(0),
                insts: vec![],
                term: Terminator {
                    opcode: 1,
                    def: None,
                    uses: vec![Operand {
                        constant: false,
                        name: Some(Name::Name("argc".to_string())),
                        ty: Type {
                            id: 13,
                            name: "i32".to_string(),
                        },
                    }],
                },
            }],
        }),
        vec![(
            HashSet::from([&Name::Name("argc".to_string())]),
            HashSet::new(),
        )],
    );
}
