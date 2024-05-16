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
    #[serde(rename = "Opcode")]
    pub opcode: usize,
    #[serde(rename = "Dest")]
    pub def: Option<Name>,
    #[serde(rename = "Uses")]
    pub uses: Vec<Operand>,
    #[serde(rename = "Blocks")]
    pub blocks: Option<Vec<Name>>,
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

pub fn cfg(
    f: &Function,
) -> (
    HashMap<&'_ Name, (&'_ BasicBlock, NodeIndex)>,
    DiGraph<&Name, ()>,
) {
    tracing::info!("cfg");
    let mut g = DiGraph::new();
    let blocks: HashMap<&'_ Name, (&'_ BasicBlock, NodeIndex)> = f
        .basic_blocks
        .iter()
        .map(|b| {
            //tracing::info!("adding {:?}", b.name);
            (&b.name, (b, g.add_node(&b.name)))
        })
        .collect();
    //tracing::info!("blocks = {blocks:?}");
    for b in &f.basic_blocks {
        //tracing::info!("term = {:?}", b.term);
        match (b.term.opcode, &b.term.uses[..]) {
            (2, &[ref l]) => {
                //tracing::info!("l = {:?}", l.name);
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

pub fn lva(f: &Function) -> Vec<(HashSet<&Name>, HashSet<&Name>)> {
    let (blocks, cfg) = cfg(f);
    tracing::info!("lva");
    let (_, block_indices) =
        f.basic_blocks
            .iter()
            .fold((f.params.len(), HashMap::new()), |(l, mut m), b| {
                m.insert(l, b);
                (l + b.insts.len() + 1, m)
            });
    //tracing::info!("{block_indices:#?}");
    let mut lives = vec![
        (HashSet::new(), HashSet::new());
        f.basic_blocks.iter().map(|b| b.insts.len() + 1).sum()
    ];
    //for bb in &f.basic_blocks {
    //tracing::info!("{}", bb.insts.len());
    //}
    for _ in 0..10 {
        for j in (0..lives.len()).rev() {
            let i = j + f.params.len();
            //tracing::info!("i = {i}");
            let (block_idx, block) = block_indices
                .iter()
                .filter(|&(j, _)| *j <= i)
                .max_by_key(|&(j, _)| *j)
                .unwrap();
            //tracing::info!("{block_idx}");

            // in[i] = use[i] U (out[i] - def[i])
            if let Some(inst) = &block.insts.get(i - (block_idx)) {
                //tracing::info!("some {i}");
                //tracing::info!(" insts[{}] = {:?}", i - block_idx, inst);
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
                //tracing::info!("use[{i}] = {:?}", r#use);
                //tracing::info!("use[{i}] = {:?}", r#use);
                lives[j].0 = r#use.union(&(&lives[j].1 - &def)).cloned().collect();
            } else {
                //tracing::info!("none");
                //tracing::info!(" term = {:?}", &block.term);
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
                //tracing::info!("use[{i}] = {:?}", r#use);
                //tracing::info!("use[{i}] = {:?}", r#use);
                lives[j].0 = r#use.union(&(&lives[j].1 - &def)).cloned().collect();
            }
            //tracing::info!(" in[{i}] = {:?}", lives[j].0);
        }

        for j in (0..lives.len()).rev() {
            let i = j + f.params.len();
            //tracing::info!("i = {i}");
            let (block_idx, block) = block_indices
                .iter()
                .filter(|&(j, _)| *j <= i)
                .max_by_key(|&(j, _)| *j)
                .unwrap();

            // out[i] = U_s=succ[i] (in[s] U phis[s])
            if let Some(_inst) = &block.insts.get(i - (block_idx)) {
                //tracing::info!("some {i}");
                // all insts only have one subsequent successor

                //tracing::info!("   setting out[{:?}] to in[{:?}]", i, i + 1);
                lives[j].1 = lives[j + 1].0.clone();
            } else {
                //tracing::info!("none");
                use petgraph::visit::IntoNodeReferences;
                // terminators must be looked up in the cfg
                //tracing::info!("searching for {:?} {:?}", block_idx, block);
                let (idx, node) = cfg
                    .node_references()
                    .find(|(_, n)| ***n == block.name)
                    .unwrap();
                //tracing::info!("{:?}", node);
                let succ: Vec<_> = cfg.neighbors(idx).collect();
                for succ in cfg.neighbors(idx) {
                    // find phis in each block
                    let name = cfg.node_weight(succ).unwrap();
                    let (source, _) = blocks.get(name).unwrap();
                    //tracing::info!("{:?}", cfg.node_weight(succ).unwrap());
                    for phi in source.insts.iter().take_while(|i| i.opcode == 55 /* phi */) {
                        //tracing::info!("phi: {:?}", phi);
                        for (source_name, uses) in
                            phi.blocks.as_ref().unwrap().iter().zip(&phi.uses)
                        {
                            //tracing::info!("source_block: {:?}", source);
                            //tracing::info!("yses: {:?}", uses);
                            //tracing::info!("{:?} {:?}", source_name, block.name);
                            if !uses.constant && *source_name == block.name {
                                tracing::info!("{:?}", (uses.name.as_ref().unwrap()));
                                lives[j].1.insert(uses.name.as_ref().unwrap());
                            }
                        }
                    }
                }
            }
            //tracing::info!("out[{i}] = {:?}", lives[j].1);
        }
        //tracing::info!("lives = {:?}", lives);
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
                },
            }],
        }),
        vec![(
            HashSet::from([&Name::Name("argc".to_string())]),
            HashSet::new(),
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
                    },
                },
            ],
        }),
        vec![
            (
                HashSet::from([&Name::Number(0)]),
                HashSet::from([&Name::Number(0), &Name::Number(3)]),
            ),
            (
                HashSet::from([&Name::Number(0), &Name::Number(3)]),
                HashSet::from([&Name::Number(0)]),
            ),
            (HashSet::from([]), HashSet::from([])),
            (
                HashSet::from([&Name::Number(0)]),
                HashSet::from([&Name::Number(0), &Name::Number(6)]),
            ),
            (
                HashSet::from([&Name::Number(0), &Name::Number(6)]),
                HashSet::from([&Name::Number(0), &Name::Number(6)]),
            ),
            (
                HashSet::from([&Name::Number(0), &Name::Number(6)]),
                HashSet::from([&Name::Number(0), &Name::Number(8)]),
            ),
            (
                HashSet::from([&Name::Number(0), &Name::Number(8)]),
                HashSet::from([&Name::Number(8), &Name::Number(9)]),
            ),
            (
                HashSet::from([&Name::Number(8), &Name::Number(9)]),
                HashSet::from([&Name::Number(8)]),
            ),
        ],
    );
}
