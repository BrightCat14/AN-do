use std::collections::HashMap;
use glob::glob;
use sha2::{Sha256, Digest};
use parser::structures::{Expr, Stmt};
use crate::hashes::hash_file;
use crate::structures::{Graph, HashStore};

pub(crate) fn build_context(parsed: &[Stmt]) -> HashMap<String, Vec<String>> {
    let mut context: HashMap<String, Vec<String>> = HashMap::new();

    for stmt in parsed {
        if let Stmt::BuildRule(br) = stmt {
            for dep in &br.deps {
                if let Expr::Call { name, args } = dep {
                    if name == "map" && args.len() >= 1 {
                        if let Expr::Word(var_name) = &args[0] {
                            if !context.contains_key(var_name) {
                                let files = glob("*").unwrap()
                                    .filter_map(|p| p.ok())
                                    .filter(|p| p.is_file())
                                    .map(|p| p.to_str().unwrap().to_string())
                                    .collect::<Vec<_>>();
                                context.insert(var_name.clone(), files);
                            }
                        }
                    }
                }
            }
        }
    }

    context
}

pub fn topo_sort(
    graph: &Graph
) -> Vec<String> {
    use std::collections::{HashSet};

    let mut visited = HashSet::new();
    let mut result = Vec::new();

    fn visit(
        name: &str,
        graph: &Graph,
        visited: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) {
        if visited.contains(name) {
            return;
        }
        visited.insert(name.to_string());

        if let Some(node) = graph.get(name) {
            for dep in &node.deps {
                visit(dep, graph, visited, result);
            }
        }

        result.push(name.to_string());
    }

    for key in graph.keys() {
        visit(key, graph, &mut visited, &mut result);
    }

    result
}

pub(crate) fn find_build_file() -> Option<String> {
    let candidates = ["ando", "ando.do", "ando.build", "do.file", "ando.file", "do.build"];

    for file in candidates {
        if std::path::Path::new(file).exists() {
            return Some(file.to_string().to_lowercase());
        }
    }

    None
}

pub fn needs_rebuild(target: &str, deps: &[String], store: &HashStore) -> bool {
    let mut combined = Sha256::new();
    for dep in deps {
        combined.update(hash_file(dep).unwrap_or_default());
    }
    let new_hash = hex::encode(combined.finalize());

    match store.hashes.get(target) {
        Some(old) => &new_hash != old,
        None => true,
    }
}
