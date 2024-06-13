use std::collections::{HashMap, HashSet};

use either::Either;
use petgraph::graph::UnGraph;

use crate::ir::{self, Function, Lva2, Name};

pub fn interf(f: &Function, lva: Lva2) -> UnGraph<Name, ()> {
    let def = ir::def(f);
    let mut g = UnGraph::new_undirected();
    let mut ids = HashMap::new();
    for (idx, (r#in, out, inst)) in lva.iter().enumerate() {
        match inst {
            Either::Left(i) => {
                for &def in &def[idx] {
                    let node1 = g.add_node(def.clone());
                    ids.insert(def, node1);

                    for out in out {
                        let node2 = g.add_node(out.clone());
                        ids.insert(out, node2);
                    }
                }
            }
            Either::Right(t) => {}
        }
    }
    g
}
