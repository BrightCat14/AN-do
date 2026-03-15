mod build;
mod system;
mod structures;
mod hashes;
mod jobs;
mod funcs;

use crate::build::build_context;
use crate::funcs::eval_map;
use crate::hashes::{load_hashes, save_hashes};
use crate::jobs::parallel_build;
use crate::structures::{Graph, Node};
use clap::Parser;
use parser::structures::{Expr, Stmt};
use parser::Lexer;
use std::collections::HashMap;
use std::process::exit;
use std::sync::{Arc, Mutex};

#[derive(Parser)]
struct Cli {
    target: Option<String>,

    #[arg(short, long)]
    jobs: Option<usize>,

    #[arg(long)]
    graph: bool,

    #[arg(long)]
    dry: bool,

    #[arg(long)]
    watch: bool,
}

fn main() {
    let cli = Cli::parse();

    // do_file reading
    let do_file: Option<String> = build::find_build_file();
    if do_file.is_none() { println!("No build file provided."); exit(1); }
    let filename = do_file.unwrap();
    let content = std::fs::read_to_string(&filename)
        .expect("Failed to read build file");

    // lex+parse
    let lex = Lexer::new(content.as_str()).lex();
    let parsed = match parser::Parser::new(lex).parse() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("parse error: {}", e);
            return;
        }
    };

    // graph
    let mut graph: Graph = HashMap::new();
    let mut rules: HashMap<String, parser::structures::RuleDef> = HashMap::new();
    let context = build_context(&parsed);

    for stmt in &parsed {
        if let Stmt::RuleDef(r) = stmt {
            rules.insert(r.name.clone(), r.clone());
        }
        if let Stmt::PatternRule(p) = stmt {
            let rule = &rules[&p.rule];
            for entry in glob::glob(&p.in_pattern).unwrap() {
                let input = entry.unwrap().to_str().unwrap().to_string();
                let base = std::path::Path::new(&input)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap();

                let output = p.out_pattern.replace("*", base);

                let command = rule.command.iter()
                    .map(|s| s.replace("$in", &input).replace("$out", &output))
                    .collect::<Vec<_>>();

                graph.insert(output.clone(), Node {
                    target: output,
                    deps: vec![input],
                    command,
                });
            }
        }
        if let Stmt::BuildRule(b) = stmt {
            let target = match &b.target {
                Expr::Word(w) => w.clone(),
                _ => panic!("only Word targets for now"),
            };
            let deps = b.deps.iter().flat_map(|dep| match dep {
                Expr::Word(w) => vec![w.clone()],
                Expr::Call { .. } => eval_map(dep, &context),
                _ => vec![],
            }).collect::<Vec<_>>();

            graph.insert(target.clone(), Node {
                target,
                deps,
                command: b.command.clone(),
            });
        }
    }


    // build
    let graph = Arc::new(graph);
    if cli.graph {
        println!("digraph G {{");
        for (target, node) in graph.iter() {
            for dep in &node.deps {
                println!("    \"{}\" -> \"{}\";", dep, target);
            }
        }
        println!("}}");
        return;
    }
    let store = Arc::new(Mutex::new(load_hashes()));
    if cli.watch {
        use notify::{recommended_watcher, RecursiveMode, Watcher};
        use std::sync::mpsc::channel;
        let (tx, rx) = channel();
        let mut watcher = recommended_watcher(tx).unwrap();
        watcher.watch(".".as_ref(), RecursiveMode::Recursive).unwrap();

        loop {
            match rx.recv() {
                Ok(event) => {
                    println!("File changed: {:?}", event);
                    parallel_build(graph.clone(), store.clone(), cli.jobs.unwrap_or(1), cli.dry);
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    } else {
        parallel_build(graph.clone(), store.clone(), cli.jobs.unwrap_or(1), cli.dry);
    }
    save_hashes(&store.lock().unwrap());
}