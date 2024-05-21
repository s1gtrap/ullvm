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

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Deserialize)]
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
    #[serde(rename = "Opcode")]
    pub opcode: usize,
    #[serde(rename = "Dest")]
    pub def: Option<Name>,
    #[serde(rename = "Uses")]
    pub uses: Vec<Operand>,
    #[serde(rename = "Blocks")]
    pub blocks: Option<Vec<Name>>,
    #[serde(rename = "String")]
    pub string: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Terminator {
    #[serde(rename = "Opcode")]
    pub opcode: usize,
    #[serde(rename = "Dest")]
    pub def: Option<Name>,
    #[serde(rename = "Uses")]
    pub uses: Vec<Operand>,
    #[serde(rename = "String")]
    pub string: String,
}

pub fn cfg(
    f: &Function,
) -> (
    HashMap<&'_ Name, (&'_ BasicBlock, NodeIndex)>,
    DiGraph<&Name, ()>,
) {
    tracing::info!("cfg {}", f.name);

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
    (blocks, g)
}

pub fn lva(f: &Function) -> Vec<(HashSet<&Name>, HashSet<&Name>, &str)> {
    tracing::info!("lva {}", f.name);

    let (blocks, cfg) = cfg(f);
    let (_, block_indices, bi): (_, _, HashMap<&Name, _>) = f.basic_blocks.iter().fold(
        (f.params.len(), HashMap::new(), HashMap::new()),
        |(l, mut m, mut n), b| {
            m.insert(l, b);
            n.insert(&b.name, l - f.params.len());
            (l + b.insts.len() + 1, m, n)
        },
    );
    let mut lives = vec![
        (HashSet::new(), HashSet::new(), "");
        f.basic_blocks.iter().map(|b| b.insts.len() + 1).sum()
    ];
    for _ in 0..10 {
        for j in (0..lives.len()).rev() {
            let i = j + f.params.len();
            let (block_idx, block) = block_indices
                .iter()
                .filter(|&(j, _)| *j <= i)
                .max_by_key(|&(j, _)| *j)
                .unwrap();

            // in[i] = use[i] U (out[i] - def[i])
            if let Some(inst) = &block.insts.get(i - (block_idx)) {
                let def = &block.insts[i - block_idx].def;
                let def: HashSet<_> = def.iter().collect();
                let r#use: HashSet<_> = if inst.opcode != 55 {
                    block.insts[i - block_idx]
                        .uses
                        .iter()
                        .filter_map(|o| {
                            if !o.constant && o.ty.id != 8 {
                                Some(o.name.as_ref().unwrap())
                            } else {
                                None
                            }
                        })
                        .collect()
                } else {
                    HashSet::new()
                };
                lives[j].0 = r#use.union(&(&lives[j].1 - &def)).cloned().collect();
                lives[j].2 = &block.insts[i - block_idx].string;
            } else {
                let def = &block.term.def;
                let def: HashSet<_> = def.iter().collect();
                let r#use: HashSet<_> = block
                    .term
                    .uses
                    .iter()
                    .filter_map(|o| {
                        if !o.constant && o.ty.id != 8 {
                            Some(o.name.as_ref().unwrap())
                        } else {
                            None
                        }
                    })
                    .collect();
                lives[j].0 = r#use.union(&(&lives[j].1 - &def)).cloned().collect();
                lives[j].2 = &block.term.string;
            }
        }

        for j in (0..lives.len()).rev() {
            let i = j + f.params.len();
            let (block_idx, block) = block_indices
                .iter()
                .filter(|&(j, _)| *j <= i)
                .max_by_key(|&(j, _)| *j)
                .unwrap();

            // out[i] = U_s=succ[i] (in[s] U phis[s])
            if let Some(_inst) = &block.insts.get(i - (block_idx)) {
                // all insts only have one subsequent successor
                lives[j].1 = lives[j + 1].0.clone();
            } else {
                use petgraph::visit::IntoNodeReferences;
                // terminators must be looked up in the cfg
                let (idx, _node) = cfg
                    .node_references()
                    .find(|(_, n)| ***n == block.name)
                    .unwrap();
                for succ in cfg.neighbors(idx) {
                    let name = cfg.node_weight(succ).unwrap();
                    let (source, _) = blocks.get(name).unwrap();

                    // copy in's from each succesor
                    lives[j].1 = lives[j]
                        .1
                        .union(&lives[bi[&source.name]].0)
                        .copied()
                        .collect();

                    // find phis in each block
                    for phi in source.insts.iter().take_while(|i| i.opcode == 55 /* phi */) {
                        for (source_name, uses) in
                            phi.blocks.as_ref().unwrap().iter().zip(&phi.uses)
                        {
                            if !uses.constant && *source_name == block.name {
                                lives[j].1.insert(uses.name.as_ref().unwrap());
                            }
                        }
                    }
                }
            }
        }
    }
    lives
}

#[test]
fn test_lva() {
    tracing_subscriber::fmt::init();

    // min.ll
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
                    string: "  ret void".to_string(),
                },
            }],
        }),
        vec![(
            HashSet::from([&Name::Name("argc".to_string())]),
            HashSet::new(),
            "  ret void",
        )],
    );
    // for1.ll
    assert_eq!(
        lva(&Function {
            name: "main".to_string(),
            params: vec![
                Param {
                    name: Name::Number(0),
                    ty: Type {
                        id: 13,
                        name: "i32".to_string(),
                    },
                },
                Param {
                    name: Name::Number(1),
                    ty: Type {
                        id: 15,
                        name: "ptr".to_string(),
                    },
                },
            ],
            basic_blocks: vec![
                BasicBlock {
                    name: Name::Number(2),
                    insts: vec![Instruction {
                        opcode: 53,
                        def: Some(Name::Number(3),),
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(Name::Number(0),),
                                ty: Type {
                                    id: 13,
                                    name: "i32".to_string(),
                                },
                            },
                            Operand {
                                constant: true,
                                name: None,
                                ty: Type {
                                    id: 13,
                                    name: "i32".to_string(),
                                },
                            },
                        ],
                        blocks: None,
                        string: "  %3 = icmp sgt i32 %0, 0".to_string(),
                    },],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(Name::Number(3),),
                                ty: Type {
                                    id: 13,
                                    name: "i1".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(Name::Number(4),),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(Name::Number(5),),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br i1 %3, label %5, label %4".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(4),
                    insts: vec![],
                    term: Terminator {
                        opcode: 1,
                        def: None,
                        uses: vec![Operand {
                            constant: true,
                            name: None,
                            ty: Type {
                                id: 13,
                                name: "i32".to_string(),
                            },
                        },],
                        string: "  ret i32 0".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(5),
                    insts: vec![
                        Instruction {
                            opcode: 55,
                            def: Some(Name::Number(6),),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(Name::Number(8),),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                            ],
                            blocks: Some(vec![Name::Number(5), Name::Number(2),],),
                            string: "  %6 = phi i32 [ %8, %5 ], [ 0, %2 ]".to_string(),
                        },
                        Instruction {
                            opcode: 56,
                            def: Some(Name::Number(7),),
                            uses: vec![
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(Name::Number(6),),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)".to_string(),
                        },
                        Instruction {
                            opcode: 13,
                            def: Some(Name::Number(8),),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(Name::Number(6),),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %8 = add nuw nsw i32 %6, 1".to_string(),
                        },
                        Instruction {
                            opcode: 53,
                            def: Some(Name::Number(9),),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(Name::Number(8),),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(Name::Number(0),),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %9 = icmp eq i32 %8, %0".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(Name::Number(9),),
                                ty: Type {
                                    id: 13,
                                    name: "i1".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(Name::Number(5),),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(Name::Number(4),),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br i1 %3, label %5, label %4".to_string(),
                    },
                },
            ],
        }),
        vec![
            (
                HashSet::from([&Name::Number(0)]),
                HashSet::from([&Name::Number(0), &Name::Number(3)]),
                "  %3 = icmp sgt i32 %0, 0",
            ),
            (
                HashSet::from([&Name::Number(0), &Name::Number(3)]),
                HashSet::from([&Name::Number(0)]),
                "  br i1 %3, label %5, label %4",
            ),
            (HashSet::from([]), HashSet::from([]), "  ret i32 0"),
            (
                HashSet::from([&Name::Number(0)]),
                HashSet::from([&Name::Number(0), &Name::Number(6)]),
                "  %6 = phi i32 [ %8, %5 ], [ 0, %2 ]",
            ),
            (
                HashSet::from([&Name::Number(0), &Name::Number(6)]),
                HashSet::from([&Name::Number(0), &Name::Number(6)]),
                "  %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)",
            ),
            (
                HashSet::from([&Name::Number(0), &Name::Number(6)]),
                HashSet::from([&Name::Number(0), &Name::Number(8)]),
                "  %8 = add nuw nsw i32 %6, 1",
            ),
            (
                HashSet::from([&Name::Number(0), &Name::Number(8)]),
                HashSet::from([&Name::Number(0), &Name::Number(8), &Name::Number(9)]),
                "  %9 = icmp eq i32 %8, %0",
            ),
            (
                HashSet::from([&Name::Number(0), &Name::Number(8), &Name::Number(9)]),
                HashSet::from([&Name::Number(0), &Name::Number(8)]),
                "  br i1 %3, label %5, label %4",
            ),
        ],
    );
}
