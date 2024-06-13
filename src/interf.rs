use std::collections::HashMap;

use petgraph::graph::UnGraph;

use crate::ir::{self, Function, Lva2, Name};

pub fn interf(f: &Function, lva: Lva2) -> UnGraph<Name, ()> {
    let def = ir::def(f);
    let mut g = UnGraph::new_undirected();
    let mut ids = HashMap::new();
    for (idx, (_in, out, _inst)) in lva.iter().enumerate() {
        for &def in &def[idx] {
            let node1 = *ids.entry(def).or_insert_with(|| g.add_node(def.clone()));

            for out in out {
                let node2 = *ids.entry(out).or_insert_with(|| g.add_node(out.clone()));
                if node1 != node2 {
                    g.add_edge(node1, node2, ());
                }
            }
        }
    }
    g
}
