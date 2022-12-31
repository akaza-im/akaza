use std::borrow::Borrow;
use std::collections::VecDeque;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};

/**
 * 簡易 LISP のインタープリタ for akaza。
 * ref. http://norvig.com/lispy.html
 */

type FunctionCallback = fn(args: VecDeque<TinyLispNode>) -> Result<TinyLispNode>;

#[derive(Clone, PartialEq, Debug)]
enum TinyLispNode {
    List(Vec<TinyLispNode>),
    String(String),
    Symbol(String),
    Function(FunctionCallback),
    LocalDateTime(DateTime<Local>),
}

fn dump_node(node: &TinyLispNode, depth: i32) {
    for _i in 0..depth {
        print!(" ");
    }

    match node {
        TinyLispNode::List(list) => {
            println!("ListNode:");
            for item in list {
                dump_node(item, depth + 1);
            }
        }
        TinyLispNode::String(s) => {
            println!("StringNode({})", s);
        }
        TinyLispNode::Symbol(s) => {
            println!("SymbolNode({})", s);
        }
        TinyLispNode::Function(_) => {
            println!("FunctionNode()");
        }
        TinyLispNode::LocalDateTime(_) => {
            println!("LocalDateTime()");
        }
    }
}

fn builtin_string_concat(args: VecDeque<TinyLispNode>) -> Result<TinyLispNode> {
    let a = &args[0];
    let b = &args[1];

    let TinyLispNode::String(a_str) = a else {
        return Err(anyhow!("argument for '.' operator should be string."));
    };
    let TinyLispNode::String(b_str) = b else {
        return Err(anyhow!("argument for '.' operator should be string."));
    };

    Ok(TinyLispNode::String(a_str.clone() + b_str))
}

fn builtin_current_datetime(_args: VecDeque<TinyLispNode>) -> Result<TinyLispNode> {
    Ok(TinyLispNode::LocalDateTime(Local::now()))
}

fn builtin_strftime(args: VecDeque<TinyLispNode>) -> Result<TinyLispNode> {
    let dt = &args[0];
    let fmt = &args[1];
    let TinyLispNode::LocalDateTime(dt) = dt else {
        return Err(anyhow!("1st argument of strftime should be LocalDateTime"));
    };
    let TinyLispNode::String(fmt) = fmt else {
        return Err(anyhow!("2nd argument of strftime should be string"));
    };
    let got = dt.format(fmt).to_string();
    Ok(TinyLispNode::String(got))
}

struct TinyLisp {}

impl TinyLisp {
    pub fn run(sexp: &str) -> Result<String> {
        let parsed = Self::parse(sexp);
        let parsed = match parsed {
            Ok(node) => node,
            Err(err) => return Err(err),
        };
        let result = Self::eval(&parsed);
        return match result {
            Ok(node) => {
                let node = node.borrow();
                if let TinyLispNode::String(ret) = node {
                    Ok(ret.clone())
                } else {
                    Err(anyhow!("Result of LISP must be String"))
                }
            }
            Err(err) => Err(err),
        };
    }

    fn parse(sexp: &str) -> Result<TinyLispNode> {
        let mut tokens = Self::tokenize(sexp);

        Self::_read_from(&mut tokens, 0)
    }

    fn eval(node: &TinyLispNode) -> Result<TinyLispNode> {
        match node {
            TinyLispNode::Symbol(symbol) => {
                if symbol == "." {
                    Ok(TinyLispNode::Function(builtin_string_concat))
                } else if symbol == "current-datetime" {
                    Ok(TinyLispNode::Function(builtin_current_datetime))
                } else if symbol == "strftime" {
                    Ok(TinyLispNode::Function(builtin_strftime))
                } else {
                    Err(anyhow!("Unknown function: {}", symbol))
                }
            }
            TinyLispNode::List(list) => {
                let mut exps: VecDeque<TinyLispNode> = VecDeque::new();
                for exp in list {
                    let result = Self::eval(exp);
                    match result {
                        Ok(node) => {
                            exps.push_back(node);
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }
                }
                let Some(proc) = exps.pop_front() else {
                    return Err(anyhow!("Empty list."));
                };
                if let TinyLispNode::Function(proc) = proc {
                    proc(exps)
                } else {
                    Err(anyhow!("Expected function... But it's not."))
                }
            }
            _ => Ok(node.clone()),
        }
    }

    fn tokenize(buf: &str) -> VecDeque<String> {
        // TODO This method should care the string literal that contains space character.
        let buf = buf.replace('(', " ( ");
        let buf = buf.replace(')', " ) ");
        let tokens: Vec<&str> = buf.split(' ').collect();
        return tokens
            .iter()
            .filter(|t| !t.is_empty())
            .map(|f| f.to_string())
            .collect();
    }

    fn _read_from(tokens: &mut VecDeque<String>, _depth: i32) -> Result<TinyLispNode> {
        if tokens.is_empty() {
            return Err(anyhow!("Unexpected EOF while reading(LISP)"));
        }

        let Some(token) = tokens.pop_front() else {
            return Err(anyhow!("Missing token... Unexpected EOS."));
        };
        if token == "(" {
            let mut values: Vec<TinyLispNode> = Vec::new();
            while tokens[0] != ")" {
                let result = Self::_read_from(tokens, _depth + 1);
                match result {
                    Ok(node) => values.push(node),
                    Err(err) => return Err(err),
                }
            }
            tokens.pop_front();
            Ok(TinyLispNode::List(values))
        } else if token == ")" {
            Err(anyhow!("Unexpected token: ')'"))
        } else {
            Ok(Self::_atom(&token))
        }
    }

    fn _atom(token: &String) -> TinyLispNode {
        return if !token.is_empty() && token.starts_with('\"') {
            TinyLispNode::String(
                token
                    .strip_prefix('\"')
                    .unwrap()
                    .strip_suffix('\"')
                    .unwrap()
                    .to_string(),
            )
        } else {
            TinyLispNode::Symbol(token.clone())
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom() {
        // symbol node
        {
            let node = TinyLisp::_atom(&"hogehoge".to_string());
            assert_eq!(node, TinyLispNode::Symbol("hogehoge".to_string()));
        }

        // string node
        {
            let node = TinyLisp::_atom(&"\"hogehoge\"".to_string());
            assert_eq!(node, TinyLispNode::String("hogehoge".to_string()));
        }
    }

    #[test]
    fn test_tokenize() {
        let tokens = TinyLisp::tokenize("(. \"a\" \"b\")");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], "(".to_string());
        assert_eq!(tokens[1], ".".to_string());
        assert_eq!(tokens[2], "\"a\"".to_string());
        assert_eq!(tokens[3], "\"b\"".to_string());
        assert_eq!(tokens[4], ")".to_string());
    }

    #[test]
    fn test_run() {
        let p = TinyLisp::run("\"hoge\"").unwrap();
        assert_eq!(p, "hoge");
    }

    #[test]
    fn test_builtin_concat() {
        let p = TinyLisp::run("(. \"h\" \"b\")").unwrap();
        assert_eq!(p, "hb");
    }

    #[test]
    fn test_builtin_strftime() {
        let src = "(strftime (current-datetime) \"%Y-%m-%d\")".to_string();
        let parsed = TinyLisp::parse(&src).unwrap();
        dump_node(&parsed, 0);
        let p = TinyLisp::run(&src).unwrap();
        assert!(p.starts_with('2')); // this test succeeds until year of 2999.
    }
}
