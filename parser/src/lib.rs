pub mod structures;

use crate::structures::{Token, Expr, Stmt, RuleDef, PatternRule, BuildRule};

pub struct Lexer {
    input: String,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer { input: input.to_string(), position: 0 }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while self.position < self.input.len() {
            let ch = self.input.as_bytes()[self.position] as char;
            if ch.is_whitespace() {
                self.position += 1;
                continue;
            }
            match ch {
                '{' => { tokens.push(Token::LBrace); self.position += 1; }
                '}' => { tokens.push(Token::RBrace); self.position += 1; }
                '(' => { tokens.push(Token::LParen); self.position += 1; }
                ')' => { tokens.push(Token::RParen); self.position += 1; }
                ',' => { tokens.push(Token::Comma); self.position += 1; }
                '"' => {
                    self.position += 1;
                    let start = self.position;
                    while self.position < self.input.len() && self.input.as_bytes()[self.position] as char != '"' {
                        self.position += 1;
                    }
                    let s = &self.input[start..self.position];
                    tokens.push(Token::String(s.to_string()));
                    if self.position < self.input.len() {
                        self.position += 1;
                    }
                }
                '$' => {
                    self.position += 1;
                    let start = self.position;
                    while self.position < self.input.len() {
                        let b = self.input.as_bytes()[self.position];
                        if b.is_ascii_alphanumeric() || b == b'_' {
                            self.position += 1;
                        } else {
                            break;
                        }
                    }
                    let s = &self.input[start..self.position];
                    tokens.push(Token::Variable(s.to_string()));
                }
                _ => {
                    let start = self.position;
                    while self.position < self.input.len() {
                        let b = self.input.as_bytes()[self.position];
                        if !b.is_ascii_whitespace() && !matches!(b as char, '{' | '}' | '(' | ')' | ',') {
                            self.position += 1;
                        } else {
                            break;
                        }
                    }
                    let word = &self.input[start..self.position];
                    match word {
                        "rule" => tokens.push(Token::KwRule),
                        "from" => tokens.push(Token::KwFrom),
                        "use" => tokens.push(Token::KwUse),
                        _ => tokens.push(Token::Word(word.to_string())),
                    }
                }
            }
        }
        tokens
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn at_eof(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn check(&self, token_types: &[Token]) -> bool {
        if self.at_eof() {
            return false;
        }
        let current_token = &self.tokens[self.current];
        token_types.iter().any(|t| t == current_token)
    }

    fn consume(&mut self) -> &Token {
        if self.at_eof() {
            panic!("Unexpected EOF");
        }
        let token = &self.tokens[self.current];
        self.current += 1;
        token
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.check(&[expected.clone()]) {
            self.consume();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, self.tokens.get(self.current)))
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while !self.at_eof() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        if self.check(&[Token::KwRule]) {
            self.parse_rule_def()
        } else if self.is_pattern_rule() {
            self.parse_pattern_rule()
        } else {
            self.parse_build_rule()
        }
    }

    fn is_pattern_rule(&self) -> bool {
        if !self.tokens.get(self.current).map(|t| matches!(t, Token::Word(_))).unwrap_or(false) {
            return false;
        }

        let pos = self.current;
        if self.tokens.get(pos + 1).map(|t| matches!(t, Token::KwFrom)).unwrap_or(false) &&
            self.tokens.get(pos + 2).map(|t| matches!(t, Token::Word(_))).unwrap_or(false) &&
            self.tokens.get(pos + 3).map(|t| matches!(t, Token::KwUse)).unwrap_or(false) {
            return true;
        }

        false
    }

    fn parse_rule_def(&mut self) -> Result<Stmt, String> {
        self.expect(Token::KwRule)?;
        let name = match self.consume() {
            Token::Word(s) => s.clone(),
            _ => return Err("Expected rule name".to_string()),
        };
        self.expect(Token::LBrace)?;
        let command = if self.check(&[Token::RBrace]) {
            Vec::new()
        } else {
            self.parse_command()?
        };
        self.expect(Token::RBrace)?;
        Ok(Stmt::RuleDef(RuleDef { name, command }))
    }

    fn parse_pattern_rule(&mut self) -> Result<Stmt, String> {
        let out_pattern = match self.consume() {
            Token::Word(s) => s.clone(),
            _ => return Err("Expected pattern".to_string()),
        };
        self.expect(Token::KwFrom)?;
        let in_pattern = match self.consume() {
            Token::Word(s) => s.clone(),
            _ => return Err("Expected pattern".to_string()),
        };
        self.expect(Token::KwUse)?;
        let rule = match self.consume() {
            Token::Word(s) => s.clone(),
            _ => return Err("Expected rule name".to_string()),
        };
        Ok(Stmt::PatternRule(PatternRule { out_pattern, in_pattern, rule }))
    }

    fn parse_build_rule(&mut self) -> Result<Stmt, String> {
        let target = self.parse_expr()?;
        self.expect(Token::KwFrom)?;

        let mut deps = Vec::new();

        if self.check(&[Token::LBrace]) {
            // nothing
        } else {
            while !self.check(&[Token::LBrace]) && !self.at_eof() {
                match self.tokens.get(self.current) {
                    Some(Token::Word(_) | Token::String(_)) => {
                        let dep = self.parse_expr()?;
                        deps.push(dep);
                    }
                    Some(Token::Comma) => { self.consume(); }
                    Some(Token::LBrace) => break,
                    Some(t) => return Err(format!("unexpected token {:?} in deps list", t)),
                    None => break,
                }
            }
        }

        self.expect(Token::LBrace)?;
        let command = self.parse_command()?;
        self.expect(Token::RBrace)?;

        Ok(Stmt::BuildRule(BuildRule { target, deps, command }))
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let token = self.tokens.get(self.current).cloned();
        if let Some(token) = token {
            match token {
                Token::Word(word) => {
                    self.current += 1;
                    if self.check(&[Token::LParen]) {
                        self.consume();
                        let mut args = Vec::new();
                        if !self.check(&[Token::RParen]) {
                            loop {
                                args.push(self.parse_expr()?);
                                if self.check(&[Token::Comma]) {
                                    self.consume();
                                } else {
                                    break;
                                }
                            }
                        }
                        self.expect(Token::RParen)?;
                        Ok(Expr::Call { name: word.clone(), args })
                    } else {
                        Ok(Expr::Word(word.clone()))
                    }
                }
                Token::String(s) => {
                    self.current += 1;
                    Ok(Expr::String(s.clone()))
                }
                _ => Err(format!("unexpected token {:?} in expression at pos {}", token, self.current)),
            }
        } else {
            Err("Unexpected EOF in expression".to_string())
        }
    }

    fn parse_command(&mut self) -> Result<Vec<String>, String> {
        let mut cmd = Vec::new();
        while !self.check(&[Token::RBrace]) && !self.at_eof() {
            let token = self.consume();
            match token {
                Token::Word(s) => cmd.push(s.clone()),
                Token::String(s) => cmd.push(s.clone()),
                Token::Variable(s) => cmd.push(format!("${}", s)),
                _ => return Err(format!("Unexpected token {:?} in command", token)),
            }
        }
        Ok(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let input = r#"
main.o from main.c {
    gcc -c main.c -o main.o
}

util.o from util.c {
    gcc -c util.c -o util.o
}

app from main.o util.o {
    gcc main.o util.o -o app
}

rule compile {
    gcc -c $in -o $out
}

rule link {
    gcc $in -o $out
}

*.o from *.c use compile

app from map(src, ".c", ".o") {
    gcc $in -o app
}
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();
        assert_eq!(stmts.len(), 7);
    }
}