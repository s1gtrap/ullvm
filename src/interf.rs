use std::collections::HashSet;

use either::Either;
use petgraph::graph::UnGraph;

use crate::ir::{Lva2, Name};

pub fn interf(lva: Lva2) -> UnGraph<Name, ()> {
    let mut g = UnGraph::new_undirected();
    for (r#in, out, inst) in lva {
        match inst {
            Either::Left(_) => {}
            Either::Right(_) => {}
        }
    }
    g
}
