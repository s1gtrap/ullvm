use std::collections::{HashMap, HashSet};
use std::fmt;

use petgraph::graph::{DiGraph, NodeIndex};

#[derive(Debug, serde::Deserialize)]
pub struct Module {
    #[serde(rename = "FunctionList")]
    pub functions: Vec<Function>,
}

#[derive(Clone, Debug, serde::Deserialize)]
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

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Type {
    #[serde(rename = "ID")]
    pub id: usize,
    #[serde(rename = "Name")]
    #[allow(dead_code)]
    pub name: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Param {
    #[serde(rename = "Name")]
    #[allow(dead_code)]
    pub name: Name,
    #[serde(rename = "Type")]
    #[allow(dead_code)]
    pub ty: Type,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct BasicBlock {
    #[serde(rename = "Name")]
    pub name: Name,
    #[serde(rename = "Instructions")]
    pub insts: Vec<Instruction>,
    #[serde(rename = "Terminator")]
    pub term: Terminator,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Operand {
    #[serde(rename = "Constant")]
    pub constant: bool,
    #[serde(rename = "Name")]
    pub name: Option<Name>,
    #[serde(rename = "Type")]
    pub ty: Type,
}

#[derive(Clone, Debug, serde::Deserialize)]
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

#[derive(Clone, Debug, serde::Deserialize)]
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

type Blocks<'a> = HashMap<&'a Name, (&'a BasicBlock, NodeIndex)>;

pub type OwnedInstLive = (HashSet<Name>, HashSet<Name>, String);

pub fn cfg(f: &Function) -> (Blocks<'_>, DiGraph<&Name, ()>) {
    tracing::trace!("cfg {}", f.name);

    let mut g = DiGraph::new();
    let blocks: HashMap<&'_ Name, (&'_ BasicBlock, NodeIndex)> = f
        .basic_blocks
        .iter()
        .map(|b| (&b.name, (b, g.add_node(&b.name))))
        .collect();
    for b in &f.basic_blocks {
        match (b.term.opcode, &b.term.uses[..]) {
            (2, [l]) => {
                g.add_edge(blocks[&b.name].1, blocks[&l.name.as_ref().unwrap()].1, ());
            }
            (2, [_, l, r]) => {
                g.add_edge(blocks[&b.name].1, blocks[&l.name.as_ref().unwrap()].1, ());
                g.add_edge(blocks[&b.name].1, blocks[&r.name.as_ref().unwrap()].1, ());
            }
            (1, _) => {}
            _ => tracing::warn!("not yet implemented: {:?}", b.term),
        }
    }
    (blocks, g)
}

pub fn def(f: &Function) -> Vec<HashSet<&Name>> {
    tracing::trace!("def {}", f.name);

    let (_blocks, _cfg) = cfg(f);
    let (_, block_indices, _bi): (_, _, HashMap<&Name, _>) = f.basic_blocks.iter().fold(
        (f.params.len(), HashMap::new(), HashMap::new()),
        |(l, mut m, mut n), b| {
            m.insert(l, b);
            n.insert(&b.name, l - f.params.len());
            (l + b.insts.len() + 1, m, n)
        },
    );
    let lives = vec![
        (HashSet::<()>::new(), HashSet::<()>::new(), "");
        f.basic_blocks.iter().map(|b| b.insts.len() + 1).sum()
    ];

    let mut defs = vec![HashSet::new(); lives.len()];

    for j in (0..lives.len()).rev() {
        let i = j + f.params.len();
        let (block_idx, block) = block_indices
            .iter()
            .filter(|&(j, _)| *j <= i)
            .max_by_key(|&(j, _)| *j)
            .unwrap();

        // in[i] = use[i] U (out[i] - def[i])
        if let Some(_inst) = &block.insts.get(i - (block_idx)) {
            let def = &block.insts[i - block_idx].def;
            let def: HashSet<_> = def.iter().collect();
            defs[j] = def;
        } else {
            let def = &block.term.def;
            let def: HashSet<_> = def.iter().collect();
            defs[j] = def;
        }
    }

    defs
}

fn block_indices(f: &Function) -> (HashMap<usize, &BasicBlock>, HashMap<&Name, usize>) {
    let (_, block_indices, bi): (_, _, HashMap<&Name, _>) = f.basic_blocks.iter().fold(
        (f.params.len(), HashMap::new(), HashMap::new()),
        |(l, mut m, mut n), b| {
            m.insert(l, b);
            n.insert(&b.name, l - f.params.len());
            (l + b.insts.len() + 1, m, n)
        },
    );
    (block_indices, bi)
}

fn init_lives(f: &Function) -> Vec<(HashSet<&Name>, HashSet<&Name>, &str)> {
    let mut lives = vec![
        (HashSet::new(), HashSet::new(), "");
        f.basic_blocks.iter().map(|b| b.insts.len() + 1).sum()
    ];
    let mut j = 0;
    for b in &f.basic_blocks {
        for i in &b.insts {
            lives[j].2 = &i.string;
            j += 1;
        }
        lives[j].2 = &b.term.string;
        j += 1;
    }

    lives
}

fn r#use(f: &Function) -> Vec<HashSet<&Name>> {
    tracing::trace!("use {}", f.name);

    let (_blocks, _cfg) = cfg(f);
    let (block_indices, _bi) = block_indices(f);
    let lives = init_lives(f);

    let mut defs = vec![HashSet::new(); lives.len()];

    for j in (0..lives.len()).rev() {
        let i = j + f.params.len();
        let (block_idx, block) = block_indices
            .iter()
            .filter(|&(j, _)| *j <= i)
            .max_by_key(|&(j, _)| *j)
            .unwrap();

        // in[i] = use[i] U (out[i] - def[i])
        if let Some(inst) = &block.insts.get(i - (block_idx)) {
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
            defs[j] = r#use;
        } else {
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
            defs[j] = r#use;
        }
    }

    defs
}

pub fn lva(f: &Function) -> Vec<(HashSet<&Name>, HashSet<&Name>, &str)> {
    tracing::trace!("lva {}", f.name);

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
                let (l, r) = lives.split_at_mut(j + 1);
                l[j].1.clone_from(&r[0].0);
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
fn test_def() {
    // min.ll
    assert_eq!(
        def(&Function {
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
        vec![HashSet::new()],
    );
    // ret.ll
    assert_eq!(
        def(&Function {
            name: "main".to_string(),
            params: vec![],
            basic_blocks: vec![BasicBlock {
                name: Name::Number(0),
                insts: vec![
                    Instruction {
                        opcode: 31,
                        def: Some(Name::Number(1),),
                        uses: vec![Operand {
                            constant: true,
                            name: None,
                            ty: Type {
                                id: 13,
                                name: "i32".to_string(),
                            },
                        },],
                        blocks: None,
                        string: "  %1 = alloca i32, align 4".to_string(),
                    },
                    Instruction {
                        opcode: 33,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: true,
                                name: None,
                                ty: Type {
                                    id: 13,
                                    name: "i32".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(Name::Number(1),),
                                ty: Type {
                                    id: 15,
                                    name: "ptr".to_string(),
                                },
                            },
                        ],
                        blocks: None,
                        string: "  store i32 0, ptr %1, align 4".to_string(),
                    },
                ],
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
                    string: "  ret i32 42".to_string(),
                },
            },],
        }),
        vec![
            HashSet::from([&Name::Number(1)]),
            HashSet::new(),
            HashSet::new()
        ]
    );
    // for0.ll
    assert_eq!(
    def(&Function {
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
                    insts: vec![
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(3),
                            ),
                            uses: vec![
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
                            string: "  %3 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(4),
                            ),
                            uses: vec![
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
                            string: "  %4 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(5),
                            ),
                            uses: vec![
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
                            string: "  %5 = alloca ptr, align 8".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(6),
                            ),
                            uses: vec![
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
                            string: "  %6 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(3),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 0, ptr %3, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(0),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(4),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 %0, ptr %4, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(1),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(5),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store ptr %1, ptr %5, align 8".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 0, ptr %6, align 4".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(7),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br label %7".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(7),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(8),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %8 = load i32, ptr %6, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(9),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(4),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %9 = load i32, ptr %4, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 53,
                            def: Some(
                                Name::Number(10),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(8),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(9),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %10 = icmp slt i32 %8, %9".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(10),
                                ),
                                ty: Type {
                                    id: 13,
                                    name: "i1".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(17),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(11),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br i1 %10, label %11, label %17".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(11),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(12),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %12 = load i32, ptr %6, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 56,
                            def: Some(
                                Name::Number(13),
                            ),
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
                                    name: Some(
                                        Name::Number(12),
                                    ),
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
                            string: "  %13 = call i32 (ptr, ...) @printf(ptr noundef @.str, i32 noundef %12)".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(14),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br label %14".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(14),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(15),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %15 = load i32, ptr %6, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 13,
                            def: Some(
                                Name::Number(16),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(15),
                                    ),
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
                            string: "  %16 = add nsw i32 %15, 1".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(16),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 %16, ptr %6, align 4".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(7),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br label %7, !llvm.loop !5".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(17),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(18),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(3),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %18 = load i32, ptr %3, align 4".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 1,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(18),
                                ),
                                ty: Type {
                                    id: 13,
                                    name: "i32".to_string(),
                                },
                            },
                        ],
                        string: "  ret i32 %18".to_string(),
                    },
                },
            ],
        }),
        vec![
            HashSet::from([&Name::Number(3)]),
            HashSet::from([&Name::Number(4)]),
            HashSet::from([&Name::Number(5)]),
            HashSet::from([&Name::Number(6)]),
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
            HashSet::from([&Name::Number(8)]),
            HashSet::from([&Name::Number(9)]),
            HashSet::from([&Name::Number(10)]),
            HashSet::new(),
            HashSet::from([&Name::Number(12)]),
            HashSet::from([&Name::Number(13)]),
            HashSet::new(),
            HashSet::from([&Name::Number(15)]),
            HashSet::from([&Name::Number(16)]),
            HashSet::new(),
            HashSet::new(),
            HashSet::from([&Name::Number(18)]),
            HashSet::new(),
        ],
    );
}

#[test]
fn test_use() {
    // min.ll
    assert_eq!(
        r#use(&Function {
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
        vec![HashSet::from([&Name::Name("argc".to_string())])],
    );
    // ret.ll
    assert_eq!(
        r#use(&Function {
            name: "main".to_string(),
            params: vec![],
            basic_blocks: vec![BasicBlock {
                name: Name::Number(0),
                insts: vec![
                    Instruction {
                        opcode: 31,
                        def: Some(Name::Number(1),),
                        uses: vec![Operand {
                            constant: true,
                            name: None,
                            ty: Type {
                                id: 13,
                                name: "i32".to_string(),
                            },
                        },],
                        blocks: None,
                        string: "  %1 = alloca i32, align 4".to_string(),
                    },
                    Instruction {
                        opcode: 33,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: true,
                                name: None,
                                ty: Type {
                                    id: 13,
                                    name: "i32".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(Name::Number(1),),
                                ty: Type {
                                    id: 15,
                                    name: "ptr".to_string(),
                                },
                            },
                        ],
                        blocks: None,
                        string: "  store i32 0, ptr %1, align 4".to_string(),
                    },
                ],
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
                    string: "  ret i32 42".to_string(),
                },
            },],
        }),
        vec![
            HashSet::new(),
            HashSet::from([&Name::Number(1)]),
            HashSet::new(),
        ]
    );
    // for0.ll
    assert_eq!(
    r#use(&Function {
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
                    insts: vec![
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(3),
                            ),
                            uses: vec![
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
                            string: "  %3 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(4),
                            ),
                            uses: vec![
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
                            string: "  %4 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(5),
                            ),
                            uses: vec![
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
                            string: "  %5 = alloca ptr, align 8".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(6),
                            ),
                            uses: vec![
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
                            string: "  %6 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(3),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 0, ptr %3, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(0),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(4),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 %0, ptr %4, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(1),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(5),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store ptr %1, ptr %5, align 8".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 0, ptr %6, align 4".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(7),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br label %7".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(7),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(8),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %8 = load i32, ptr %6, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(9),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(4),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %9 = load i32, ptr %4, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 53,
                            def: Some(
                                Name::Number(10),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(8),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(9),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %10 = icmp slt i32 %8, %9".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(10),
                                ),
                                ty: Type {
                                    id: 13,
                                    name: "i1".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(17),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(11),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br i1 %10, label %11, label %17".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(11),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(12),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %12 = load i32, ptr %6, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 56,
                            def: Some(
                                Name::Number(13),
                            ),
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
                                    name: Some(
                                        Name::Number(12),
                                    ),
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
                            string: "  %13 = call i32 (ptr, ...) @printf(ptr noundef @.str, i32 noundef %12)".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(14),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br label %14".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(14),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(15),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %15 = load i32, ptr %6, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 13,
                            def: Some(
                                Name::Number(16),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(15),
                                    ),
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
                            string: "  %16 = add nsw i32 %15, 1".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(16),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 %16, ptr %6, align 4".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(7),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br label %7, !llvm.loop !5".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(17),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(18),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(3),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %18 = load i32, ptr %3, align 4".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 1,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(18),
                                ),
                                ty: Type {
                                    id: 13,
                                    name: "i32".to_string(),
                                },
                            },
                        ],
                        string: "  ret i32 %18".to_string(),
                    },
                },
            ],
        }),
        vec![
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
            HashSet::from([&Name::Number(3)]),
            HashSet::from([&Name::Number(0),&Name::Number(4)]),
            HashSet::from([&Name::Number(1),&Name::Number(5)]),
            HashSet::from([&Name::Number(6)]),
            HashSet::new(),
            HashSet::from([&Name::Number(6)]),
            HashSet::from([&Name::Number(4)]),
            HashSet::from([&Name::Number(8),&Name::Number(9)]),
            HashSet::from([&Name::Number(10)]),
            HashSet::from([&Name::Number(6)]),
            HashSet::from([&Name::Number(12)]),
            HashSet::new(),
            HashSet::from([&Name::Number(6)]),
            HashSet::from([&Name::Number(15)]),
            HashSet::from([&Name::Number(6),&Name::Number(16)]),
            HashSet::new(),
            HashSet::from([&Name::Number(3)]),
            HashSet::from([&Name::Number(18)]),
        ],
    );
}

pub enum IterState {
    In,
    Out,
}

pub struct Iter {
    //inner: IterInner<'a, 'b, I>,
    state: IterState,
    f: Function,
    blocks: HashMap<Name, (BasicBlock, NodeIndex)>,
    cfg: DiGraph<Name, ()>,
    lives: Vec<(HashSet<Name>, HashSet<Name>, String)>,
    lives_clone: Vec<(HashSet<Name>, HashSet<Name>, String)>,
    block_indices: HashMap<usize, BasicBlock>,
    bi: HashMap<Name, usize>,
    r#use: Vec<HashSet<Name>>,
    def: Vec<HashSet<Name>>,
    index_iter: std::iter::Rev<std::ops::Range<usize>>,
}

impl Iter {
    pub fn new(f: &Function) -> Self {
        let f = f.clone();
        let lives = init_lives(&f);
        let iter = (0..lives.len()).rev();
        let r#use = r#use(&f)
            .iter()
            .map(|u| u.iter().cloned().cloned().collect())
            .collect();
        let def = def(&f)
            .iter()
            .map(|d| d.iter().cloned().cloned().collect())
            .collect();
        let (blocks, cfg) = cfg(&f);
        let (_, block_indices, bi): (_, _, HashMap<&Name, _>) = f.basic_blocks.iter().fold(
            (f.params.len(), HashMap::new(), HashMap::new()),
            |(l, mut m, mut n), b| {
                m.insert(l, b);
                n.insert(&b.name, l - f.params.len());
                (l + b.insts.len() + 1, m, n)
            },
        );
        let lives: Vec<_> = lives
            .iter()
            .map(|(i, o, s)| {
                (
                    i.iter().map(|&i| i.clone()).collect(),
                    o.iter().map(|&o| o.clone()).collect(),
                    s.to_string(),
                )
            })
            .collect();
        let blocks = blocks
            .iter()
            .map(|(&k, &(b, n))| (k.clone(), (b.clone(), n)))
            .collect();
        let cfg = cfg.map(|_, &n| n.clone(), |_, _| ());
        let block_indices = block_indices
            .iter()
            .map(|(&k, &v)| (k, v.clone()))
            .collect();
        let bi = bi.iter().map(|(&k, &v)| (k.clone(), v)).collect();
        Iter {
            state: IterState::In,
            f,
            blocks,
            cfg,
            lives_clone: lives.to_vec(),
            lives,
            block_indices,
            bi,
            r#use,
            def,
            index_iter: iter,
            //inner: IterInner::In(InIter::new(f, lives, iter)),
        }
    }
}

impl Iterator for Iter {
    type Item = Vec<(HashSet<Name>, HashSet<Name>, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            IterState::In => {
                if let Some(j) = self.index_iter.next() {
                    // in[i] = use[i] U (out[i] - def[i])
                    let def = &self.def[j];
                    let r#use = &self.r#use[j];
                    self.lives[j].0 = r#use.union(&(&self.lives[j].1 - def)).cloned().collect();
                    Some(self.lives.clone())
                } else {
                    self.state = IterState::Out;
                    self.index_iter = (0..self.lives.len()).rev();
                    self.next()
                }
            }
            IterState::Out => {
                if let Some(j) = self.index_iter.next() {
                    let i = j + self.f.params.len();
                    let (block_idx, block) = self
                        .block_indices
                        .iter()
                        .filter(|&(j, _)| *j <= i)
                        .max_by_key(|&(j, _)| *j)
                        .unwrap();

                    // out[i] = U_s=succ[i] (in[s] U phis[s])
                    if let Some(_inst) = &block.insts.get(i - (block_idx)) {
                        // all insts only have one subsequent successor
                        let (l, r) = self.lives.split_at_mut(j + 1);
                        l[j].1.clone_from(&r[0].0);
                    } else {
                        use petgraph::visit::IntoNodeReferences;
                        // terminators must be looked up in the cfg
                        let (idx, _node) = self
                            .cfg
                            .node_references()
                            .find(|(_, n)| **n == block.name)
                            .unwrap();
                        for succ in self.cfg.neighbors(idx) {
                            let name = self.cfg.node_weight(succ).unwrap();
                            let (source, _) = self.blocks.get(name).unwrap();

                            // copy in's from each succesor
                            self.lives[j].1 = self.lives[j]
                                .1
                                .union(&self.lives[self.bi[&source.name]].0)
                                .cloned()
                                .collect();

                            // find phis in each block
                            for phi in
                                source.insts.iter().take_while(|i| i.opcode == 55 /* phi */)
                            {
                                for (source_name, uses) in
                                    phi.blocks.as_ref().unwrap().iter().zip(&phi.uses)
                                {
                                    if !uses.constant && *source_name == block.name {
                                        self.lives[j].1.insert(uses.name.clone().unwrap());
                                    }
                                }
                            }
                        }
                    }
                    Some(self.lives.clone())
                } else {
                    self.state = IterState::In;
                    self.index_iter = (0..self.lives.len()).rev();
                    if self.lives != self.lives_clone {
                        self.lives_clone.clone_from(&self.lives);
                        Some(self.lives.clone())
                    } else {
                        None
                    }
                }
            }
        }
    }
}

#[test]
fn test_iter() {
    //use pretty_assertions::assert_eq;

    // ret.ll
    let f = Function {
        name: "main".to_string(),
        params: vec![],
        basic_blocks: vec![BasicBlock {
            name: Name::Number(0),
            insts: vec![
                Instruction {
                    opcode: 31,
                    def: Some(Name::Number(1)),
                    uses: vec![Operand {
                        constant: true,
                        name: None,
                        ty: Type {
                            id: 13,
                            name: "i32".to_string(),
                        },
                    }],
                    blocks: None,
                    string: "  %1 = alloca i32, align 4".to_string(),
                },
                Instruction {
                    opcode: 33,
                    def: None,
                    uses: vec![
                        Operand {
                            constant: true,
                            name: None,
                            ty: Type {
                                id: 13,
                                name: "i32".to_string(),
                            },
                        },
                        Operand {
                            constant: false,
                            name: Some(Name::Number(1)),
                            ty: Type {
                                id: 15,
                                name: "ptr".to_string(),
                            },
                        },
                    ],
                    blocks: None,
                    string: "  store i32 0, ptr %1, align 4".to_string(),
                },
            ],
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
                }],
                string: "  ret i32 42".to_string(),
            },
        }],
    };
    assert_eq!(
        Iter::new(&f).collect::<Vec<_>>(),
        vec![
            vec![
                (
                    HashSet::new(),
                    HashSet::new(),
                    "  %1 = alloca i32, align 4".to_string()
                ),
                (
                    HashSet::new(),
                    HashSet::new(),
                    "  store i32 0, ptr %1, align 4".to_string(),
                ),
                (HashSet::new(), HashSet::new(), "  ret i32 42".to_string()),
            ],
            vec![
                (
                    HashSet::new(),
                    HashSet::new(),
                    "  %1 = alloca i32, align 4".to_string()
                ),
                (
                    HashSet::from([Name::Number(1)]),
                    HashSet::new(),
                    "  store i32 0, ptr %1, align 4".to_string(),
                ),
                (HashSet::new(), HashSet::new(), "  ret i32 42".to_string()),
            ],
            vec![
                (
                    HashSet::new(),
                    HashSet::new(),
                    "  %1 = alloca i32, align 4".to_string()
                ),
                (
                    HashSet::from([Name::Number(1)]),
                    HashSet::new(),
                    "  store i32 0, ptr %1, align 4".to_string(),
                ),
                (HashSet::new(), HashSet::new(), "  ret i32 42".to_string()),
            ],
            vec![
                (
                    HashSet::new(),
                    HashSet::new(),
                    "  %1 = alloca i32, align 4".to_string()
                ),
                (
                    HashSet::from([Name::Number(1)]),
                    HashSet::new(),
                    "  store i32 0, ptr %1, align 4".to_string(),
                ),
                (HashSet::new(), HashSet::new(), "  ret i32 42".to_string()),
            ],
            vec![
                (
                    HashSet::new(),
                    HashSet::new(),
                    "  %1 = alloca i32, align 4".to_string()
                ),
                (
                    HashSet::from([Name::Number(1)]),
                    HashSet::new(),
                    "  store i32 0, ptr %1, align 4".to_string(),
                ),
                (HashSet::new(), HashSet::new(), "  ret i32 42".to_string()),
            ],
            vec![
                (
                    HashSet::new(),
                    HashSet::from([Name::Number(1)]),
                    "  %1 = alloca i32, align 4".to_string(),
                ),
                (
                    HashSet::from([Name::Number(1)]),
                    HashSet::new(),
                    "  store i32 0, ptr %1, align 4".to_string(),
                ),
                (HashSet::new(), HashSet::new(), "  ret i32 42".to_string()),
            ],
            // FIXME: continues for 2n iterations w/o changes
            vec![
                (
                    HashSet::new(),
                    HashSet::from([Name::Number(1)]),
                    "  %1 = alloca i32, align 4".to_string(),
                ),
                (
                    HashSet::from([Name::Number(1)]),
                    HashSet::new(),
                    "  store i32 0, ptr %1, align 4".to_string(),
                ),
                (HashSet::new(), HashSet::new(), "  ret i32 42".to_string()),
            ],
            vec![
                (
                    HashSet::new(),
                    HashSet::from([Name::Number(1)]),
                    "  %1 = alloca i32, align 4".to_string(),
                ),
                (
                    HashSet::from([Name::Number(1)]),
                    HashSet::new(),
                    "  store i32 0, ptr %1, align 4".to_string(),
                ),
                (HashSet::new(), HashSet::new(), "  ret i32 42".to_string()),
            ],
            vec![
                (
                    HashSet::new(),
                    HashSet::from([Name::Number(1)]),
                    "  %1 = alloca i32, align 4".to_string(),
                ),
                (
                    HashSet::from([Name::Number(1)]),
                    HashSet::new(),
                    "  store i32 0, ptr %1, align 4".to_string(),
                ),
                (HashSet::new(), HashSet::new(), "  ret i32 42".to_string()),
            ],
            vec![
                (
                    HashSet::new(),
                    HashSet::from([Name::Number(1)]),
                    "  %1 = alloca i32, align 4".to_string(),
                ),
                (
                    HashSet::from([Name::Number(1)]),
                    HashSet::new(),
                    "  store i32 0, ptr %1, align 4".to_string(),
                ),
                (HashSet::new(), HashSet::new(), "  ret i32 42".to_string()),
            ],
            vec![
                (
                    HashSet::new(),
                    HashSet::from([Name::Number(1)]),
                    "  %1 = alloca i32, align 4".to_string(),
                ),
                (
                    HashSet::from([Name::Number(1)]),
                    HashSet::new(),
                    "  store i32 0, ptr %1, align 4".to_string(),
                ),
                (HashSet::new(), HashSet::new(), "  ret i32 42".to_string()),
            ],
            vec![
                (
                    HashSet::new(),
                    HashSet::from([Name::Number(1)]),
                    "  %1 = alloca i32, align 4".to_string(),
                ),
                (
                    HashSet::from([Name::Number(1)]),
                    HashSet::new(),
                    "  store i32 0, ptr %1, align 4".to_string(),
                ),
                (HashSet::new(), HashSet::new(), "  ret i32 42".to_string()),
            ],
            vec![
                (
                    HashSet::new(),
                    HashSet::from([Name::Number(1)]),
                    "  %1 = alloca i32, align 4".to_string(),
                ),
                (
                    HashSet::from([Name::Number(1)]),
                    HashSet::new(),
                    "  store i32 0, ptr %1, align 4".to_string(),
                ),
                (HashSet::new(), HashSet::new(), "  ret i32 42".to_string()),
            ],
        ],
    );

    // for0.ll
    let f = Function {
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
                    insts: vec![
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(3),
                            ),
                            uses: vec![
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
                            string: "  %3 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(4),
                            ),
                            uses: vec![
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
                            string: "  %4 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(5),
                            ),
                            uses: vec![
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
                            string: "  %5 = alloca ptr, align 8".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(6),
                            ),
                            uses: vec![
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
                            string: "  %6 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(3),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 0, ptr %3, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(0),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(4),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 %0, ptr %4, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(1),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(5),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store ptr %1, ptr %5, align 8".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 0, ptr %6, align 4".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(7),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br label %7".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(7),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(8),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %8 = load i32, ptr %6, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(9),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(4),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %9 = load i32, ptr %4, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 53,
                            def: Some(
                                Name::Number(10),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(8),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(9),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %10 = icmp slt i32 %8, %9".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(10),
                                ),
                                ty: Type {
                                    id: 13,
                                    name: "i1".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(17),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(11),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br i1 %10, label %11, label %17".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(11),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(12),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %12 = load i32, ptr %6, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 56,
                            def: Some(
                                Name::Number(13),
                            ),
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
                                    name: Some(
                                        Name::Number(12),
                                    ),
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
                            string: "  %13 = call i32 (ptr, ...) @printf(ptr noundef @.str, i32 noundef %12)".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(14),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br label %14".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(14),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(15),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %15 = load i32, ptr %6, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 13,
                            def: Some(
                                Name::Number(16),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(15),
                                    ),
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
                            string: "  %16 = add nsw i32 %15, 1".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(16),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 %16, ptr %6, align 4".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(7),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br label %7, !llvm.loop !5".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(17),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(18),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(3),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %18 = load i32, ptr %3, align 4".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 1,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(18),
                                ),
                                ty: Type {
                                    id: 13,
                                    name: "i32".to_string(),
                                },
                            },
                        ],
                        string: "  ret i32 %18".to_string(),
                    },
                },
            ],
        };
    assert_eq!(
        Iter::new(&f)
            .map(|i| i
                .iter()
                .map(|(r#in, out, _)| (r#in.clone(), out.clone()))
                .collect::<Vec<_>>())
            .take(45)
            .collect::<Vec<_>>(),
        vec![
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::new()),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)]),
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(15)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(10)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::from([Name::Number(4)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::from([Name::Number(10)])
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(4)]),
                    HashSet::from([Name::Number(8), Name::Number(9)])
                ),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::from([Name::Number(10)])
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(4)]),
                    HashSet::from([Name::Number(8), Name::Number(9)])
                ),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::from([Name::Number(10)])
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(4)]),
                    HashSet::from([Name::Number(8), Name::Number(9)])
                ),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::from([Name::Number(10)])
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::new()
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(4)]),
                    HashSet::from([Name::Number(8), Name::Number(9)])
                ),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::from([Name::Number(10)])
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::new()
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::from([Name::Number(6)])
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(4)]),
                    HashSet::from([Name::Number(8), Name::Number(9)])
                ),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::from([Name::Number(10)])
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::from([Name::Number(3)]), HashSet::new()),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::from([Name::Number(1), Name::Number(5)])
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::from([Name::Number(6)])
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(4)]),
                    HashSet::from([Name::Number(8), Name::Number(9)])
                ),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::from([Name::Number(10)])
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(0), Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::from([Name::Number(1), Name::Number(5)])
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::from([Name::Number(6)])
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(4)]),
                    HashSet::from([Name::Number(8), Name::Number(9)])
                ),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::from([Name::Number(10)])
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(3)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(0), Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::from([Name::Number(1), Name::Number(5)])
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::from([Name::Number(6)])
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(4)]),
                    HashSet::from([Name::Number(8), Name::Number(9)])
                ),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::from([Name::Number(10)])
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(3)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(0), Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::from([Name::Number(1), Name::Number(5)])
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::from([Name::Number(6)])
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(4)]),
                    HashSet::from([Name::Number(8), Name::Number(9)])
                ),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::from([Name::Number(10)])
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(3)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(0), Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::from([Name::Number(1), Name::Number(5)])
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::from([Name::Number(6)])
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(4)]),
                    HashSet::from([Name::Number(8), Name::Number(9)])
                ),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::from([Name::Number(10)])
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(3)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(0), Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::from([Name::Number(1), Name::Number(5)])
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::from([Name::Number(6)])
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(4)]),
                    HashSet::from([Name::Number(8), Name::Number(9)])
                ),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::from([Name::Number(10)])
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
            vec![
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(3)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(0), Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(0), Name::Number(4)]),
                    HashSet::from([Name::Number(1), Name::Number(5)])
                ),
                (
                    HashSet::from([Name::Number(1), Name::Number(5)]),
                    HashSet::from([Name::Number(6)])
                ),
                (HashSet::from([Name::Number(6)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(4)])
                ),
                (
                    HashSet::from([Name::Number(4)]),
                    HashSet::from([Name::Number(8), Name::Number(9)])
                ),
                (
                    HashSet::from([Name::Number(8), Name::Number(9)]),
                    HashSet::from([Name::Number(10)])
                ),
                (
                    HashSet::from([Name::Number(10)]),
                    HashSet::from([Name::Number(6), Name::Number(3)])
                ),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(12)])
                ),
                (HashSet::from([Name::Number(12)]), HashSet::new()),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(6)]),
                    HashSet::from([Name::Number(15)])
                ),
                (
                    HashSet::from([Name::Number(15)]),
                    HashSet::from([Name::Number(6), Name::Number(16)])
                ),
                (
                    HashSet::from([Name::Number(6), Name::Number(16)]),
                    HashSet::new()
                ),
                (HashSet::new(), HashSet::from([Name::Number(6)])),
                (
                    HashSet::from([Name::Number(3)]),
                    HashSet::from([Name::Number(18)])
                ),
                (HashSet::from([Name::Number(18)]), HashSet::new()),
            ],
        ],
    );
}

#[test]
fn test_lva() {
    use pretty_assertions::assert_eq;

    let _ = tracing_subscriber::fmt::try_init();

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
    /*// fib.ll
    assert_eq!(
        lva(&Function {
            name: "main".to_string(),
            params: vec![],
            basic_blocks: vec![
                BasicBlock {
                    name: Name::Number(0),
                    insts: vec![
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(1),
                            ),
                            uses: vec![
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
                            string: "  %1 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(2),
                            ),
                            uses: vec![
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
                            string: "  %2 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(3),
                            ),
                            uses: vec![
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
                            string: "  %3 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(4),
                            ),
                            uses: vec![
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
                            string: "  %4 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(5),
                            ),
                            uses: vec![
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
                            string: "  %5 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 31,
                            def: Some(
                                Name::Number(6),
                            ),
                            uses: vec![
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
                            string: "  %6 = alloca i32, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(1),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 0, ptr %1, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(4),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 0, ptr %4, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(5),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 1, ptr %5, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(7),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(4),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %7 = load i32, ptr %4, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(8),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(5),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %8 = load i32, ptr %5, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 13,
                            def: Some(
                                Name::Number(9),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(7),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(8),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %9 = add nsw i32 %7, %8".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(9),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 %9, ptr %6, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 56,
                            def: Some(
                                Name::Number(10),
                            ),
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
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %10 = call i32 (ptr, ...) @printf(ptr noundef @.str)".to_string(),
                        },
                        Instruction {
                            opcode: 56,
                            def: Some(
                                Name::Number(11),
                            ),
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
                                    name: Some(
                                        Name::Number(3),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
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
                            string: "  %11 = call i32 (ptr, ...) @scanf(ptr noundef @.str.1, ptr noundef %3)".to_string(),
                        },
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(12),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(4),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %12 = load i32, ptr %4, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(13),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(5),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %13 = load i32, ptr %5, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 56,
                            def: Some(
                                Name::Number(14),
                            ),
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
                                    name: Some(
                                        Name::Number(12),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(13),
                                    ),
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
                            string: "  %14 = call i32 (ptr, ...) @printf(ptr noundef @.str.2, i32 noundef %12, i32 noundef %13)".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: true,
                                    name: None,
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(2),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 3, ptr %2, align 4".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(15),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br label %15".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(15),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(16),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(2),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %16 = load i32, ptr %2, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(17),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(3),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %17 = load i32, ptr %3, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 53,
                            def: Some(
                                Name::Number(18),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(16),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(17),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %18 = icmp sle i32 %16, %17".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(18),
                                ),
                                ty: Type {
                                    id: 13,
                                    name: "i1".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(30),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(19),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br i1 %18, label %19, label %30".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(19),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(20),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %20 = load i32, ptr %6, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 56,
                            def: Some(
                                Name::Number(21),
                            ),
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
                                    name: Some(
                                        Name::Number(20),
                                    ),
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
                            string: "  %21 = call i32 (ptr, ...) @printf(ptr noundef @.str.3, i32 noundef %20)".to_string(),
                        },
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(22),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(5),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %22 = load i32, ptr %5, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(22),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(4),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 %22, ptr %4, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(23),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %23 = load i32, ptr %6, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(23),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(5),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 %23, ptr %5, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(24),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(4),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %24 = load i32, ptr %4, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(25),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(5),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %25 = load i32, ptr %5, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 13,
                            def: Some(
                                Name::Number(26),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(24),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(25),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %26 = add nsw i32 %24, %25".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(26),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(6),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 %26, ptr %6, align 4".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(27),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br label %27".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(27),
                    insts: vec![
                        Instruction {
                            opcode: 32,
                            def: Some(
                                Name::Number(28),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(2),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  %28 = load i32, ptr %2, align 4".to_string(),
                        },
                        Instruction {
                            opcode: 13,
                            def: Some(
                                Name::Number(29),
                            ),
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(28),
                                    ),
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
                            string: "  %29 = add nsw i32 %28, 1".to_string(),
                        },
                        Instruction {
                            opcode: 33,
                            def: None,
                            uses: vec![
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(29),
                                    ),
                                    ty: Type {
                                        id: 13,
                                        name: "i32".to_string(),
                                    },
                                },
                                Operand {
                                    constant: false,
                                    name: Some(
                                        Name::Number(2),
                                    ),
                                    ty: Type {
                                        id: 15,
                                        name: "ptr".to_string(),
                                    },
                                },
                            ],
                            blocks: None,
                            string: "  store i32 %29, ptr %2, align 4".to_string(),
                        },
                    ],
                    term: Terminator {
                        opcode: 2,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: false,
                                name: Some(
                                    Name::Number(15),
                                ),
                                ty: Type {
                                    id: 8,
                                    name: "label".to_string(),
                                },
                            },
                        ],
                        string: "  br label %15, !llvm.loop !5".to_string(),
                    },
                },
                BasicBlock {
                    name: Name::Number(30),
                    insts: vec![],
                    term: Terminator {
                        opcode: 1,
                        def: None,
                        uses: vec![
                            Operand {
                                constant: true,
                                name: None,
                                ty: Type {
                                    id: 13,
                                    name: "i32".to_string(),
                                },
                            },
                        ],
                        string: "  ret i32 0".to_string(),
                    },
                },
            ],
        }),
        vec![
            (
                HashSet::from([]),
                HashSet::from([&Name::Number(1)]),
                "  %1 = alloca i32, align 4",
            ),
            (
                HashSet::from([&Name::Number(1)]),
                HashSet::from([&Name::Number(1), &Name::Number(2)]),
                "  %2 = alloca i32, align 4",
            ),
            (HashSet::from([]), HashSet::from([]), "  ret i32 0"),
            (
                HashSet::from([&Name::Number(1), &Name::Number(2)]),
                HashSet::from([&Name::Number(1), &Name::Number(2), &Name::Number(3)]),
                "  %3 = alloca i32, align 4",
            ),
            (
                HashSet::from([&Name::Number(1), &Name::Number(2), &Name::Number(3)]),
                HashSet::from([
                    &Name::Number(1),
                    &Name::Number(2),
                    &Name::Number(3),
                    &Name::Number(4),
                ]),
                "  %4 = alloca i32, align 4",
            ),
            (
                HashSet::from([
                    &Name::Number(1),
                    &Name::Number(2),
                    &Name::Number(3),
                    &Name::Number(4),
                ]),
                HashSet::from([
                    &Name::Number(1),
                    &Name::Number(2),
                    &Name::Number(3),
                    &Name::Number(4),
                    &Name::Number(5),
                ]),
                "  %5 = alloca i32, align 4",
            ),
            (
                HashSet::from([
                    &Name::Number(1),
                    &Name::Number(2),
                    &Name::Number(3),
                    &Name::Number(4),
                    &Name::Number(5),
                ]),
                HashSet::from([
                    &Name::Number(1),
                    &Name::Number(2),
                    &Name::Number(3),
                    &Name::Number(4),
                    &Name::Number(5),
                    &Name::Number(6),
                ]),
                "  %6 = alloca i32, align 4",
            ),
            ( HashSet::from([&Name::Number(1),]), HashSet::from([]), "  store i32 0, ptr %1, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  store i32 0, ptr %4, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  store i32 1, ptr %5, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %7 = load i32, ptr %4, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %8 = load i32, ptr %5, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %9 = add nsw i32 %7, %8",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  store i32 %9, ptr %6, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %10 = call i32 (ptr, ...) @printf(ptr noundef @.str)",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %11 = call i32 (ptr, ...) @scanf(ptr noundef @.str.1, ptr noundef %3)",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %12 = load i32, ptr %4, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %13 = load i32, ptr %5, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %14 = call i32 (ptr, ...) @printf(ptr noundef @.str.2, i32 noundef %12, i32 noundef %13)",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  store i32 3, ptr %2, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  br label %15",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %16 = load i32, ptr %2, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %17 = load i32, ptr %3, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %18 = icmp sle i32 %16, %17",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  br i1 %18, label %19, label %30",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %20 = load i32, ptr %6, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %21 = call i32 (ptr, ...) @printf(ptr noundef @.str.3, i32 noundef %20)",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %22 = load i32, ptr %5, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  store i32 %22, ptr %4, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %23 = load i32, ptr %6, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  store i32 %23, ptr %5, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %24 = load i32, ptr %4, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %25 = load i32, ptr %5, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %26 = add nsw i32 %24, %25",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  store i32 %26, ptr %6, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  br label %27",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %28 = load i32, ptr %2, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  %29 = add nsw i32 %28, 1",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  store i32 %29, ptr %2, align 4",),
            ( HashSet::from([&Name::Number(2),]), HashSet::from([&Name::Number(2),]), "  br label %15, !llvm.loop !5",),
            ( HashSet::from([]), HashSet::from([]), "  ret i32 0",),
        ],
    );*/
}
