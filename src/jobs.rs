use crate::build::{needs_rebuild, topo_sort};
use crate::hashes::update_hash;
use crate::structures::{Graph, HashStore};
use crate::system::run_command;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub fn parallel_build(
    graph: Arc<Graph>,
    store: Arc<Mutex<HashStore>>,
    jobs: usize,
    dry: bool
) {
    let build_order = topo_sort(&*graph);

    let graph = Arc::clone(&graph);

    build_order.into_par_iter()
        .with_max_len(jobs)
        .for_each(|target| {
            if let Some(node) = graph.get(&target) {
                let store_lock = store.lock().unwrap();
                let rebuild = needs_rebuild(&node.target, &node.deps, &store_lock);
                drop(store_lock);

                if rebuild {
                    if dry {
                        let command = &node.command.join(" ");
                        if command.is_empty() { } else {
                            println!("dry run: would build {}", target);
                            println!("command: {:?}", command);
                        }
                    } else {
                        println!("building {}", target);
                        run_command(&node.command);

                        let mut store_lock = store.lock().unwrap();
                        update_hash(&node.target, &node.deps, &mut store_lock);
                    }
                } else {
                    println!("{} is up-to-date", target);
                }
            } else {
                println!("warning: target {} not found in graph", target);
            }
        });
}