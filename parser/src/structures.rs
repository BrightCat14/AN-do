#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Word(String),
    String(String),
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    String(String),
    Variable(String),
    LBrace,
    RBrace,
    LParen,
    RParen,
    Comma,
    KwRule,
    KwFrom,
    KwUse,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RuleDef {
    pub name: String,
    pub command: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PatternRule {
    pub out_pattern: String,
    pub in_pattern: String,
    pub rule: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BuildRule {
    pub target: Expr,
    pub deps: Vec<Expr>,
    pub command: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    RuleDef(RuleDef),
    PatternRule(PatternRule),
    BuildRule(BuildRule),
}