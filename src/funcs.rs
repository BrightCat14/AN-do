use std::collections::HashMap;
use parser::structures::Expr;

pub fn eval_map(expr: &Expr, context: &HashMap<String, Vec<String>>) -> Vec<String> {
    if let Expr::Call { name, args } = expr {
        if name == "map" && args.len() == 3 {
            if let Expr::Word(var_name) = &args[0] {
                let from_ext = if let Expr::String(s) = &args[1] { s } else { "" };
                let to_ext = if let Expr::String(s) = &args[2] { s } else { "" };

                if let Some(files) = context.get(var_name) {
                    return files.iter()
                        .map(|f| f.replace(from_ext, to_ext))
                        .collect();
                }
            }
        }
    }
    vec![]
}