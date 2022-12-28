use std::borrow::Borrow;
use std::collections::VecDeque;

use chrono::{DateTime, Local};

/**
 * 簡易 LISP のインタープリタ for akaza。
 * ref. http://norvig.com/lispy.html
 */

type FunctionCallback = fn(args: VecDeque<Node>) -> Result<Node, String>;

#[derive(Clone, PartialEq, Debug)]
enum Node {
    ListNode(Vec<Node>),
    StringNode(String),
    SymbolNode(String),
    FunctionNode(FunctionCallback),
    LocalDateTimeNode(DateTime<Local>),
}

fn dump_node(node: &Node, depth: i32) {
    for _i in 0..depth {
        print!(" ");
    }

    match node {
        Node::ListNode(list) => {
            println!("ListNode:");
            for item in list {
                dump_node(item, depth + 1);
            }
        }
        Node::StringNode(s) => {
            println!("StringNode({})", s);
        }
        Node::SymbolNode(s) => {
            println!("SymbolNode({})", s);
        }
        Node::FunctionNode(_) => {
            println!("FunctionNode()");
        }
        Node::LocalDateTimeNode(_) => {
            println!("LocalDateTime()");
        }
    }
}

fn builtin_string_concat(args: VecDeque<Node>) -> Result<Node, String> {
    let a = &args[0];
    let b = &args[1];

    let Node::StringNode(a_str) = a else {
        return Err("argument for '.' operator should be string.".to_string());
    };
    let Node::StringNode(b_str) = b else {
        return Err("argument for '.' operator should be string.".to_string());
    };

    return Ok(Node::StringNode(a_str.clone() + b_str));
}

fn builtin_current_datetime(_args: VecDeque<Node>) -> Result<Node, String> {
    return Ok(Node::LocalDateTimeNode(Local::now()));
}

fn builtin_strftime(args: VecDeque<Node>) -> Result<Node, String> {
    let dt = &args[0];
    let fmt = &args[1];
    let Node::LocalDateTimeNode(dt) = dt else {
        return Err("1st argument of strftime should be LocalDateTime".to_string());
    };
    let Node::StringNode(fmt) = fmt else {
        return Err("2nd argument of strftime should be string".to_string());
    };
    let got = dt.format(fmt).to_string();
    return Ok(Node::StringNode(got));
}

struct TinyLisp {}

impl TinyLisp {
    pub fn run(sexp: &String) -> Result<String, String> {
        let parsed = Self::parse(sexp);
        let parsed = match parsed {
            Ok(node) => node,
            Err(err) => return Err(err),
        };
        let result = Self::eval(&parsed);
        return match result {
            Ok(node) => {
                let node = node.borrow();
                if let Node::StringNode(ret) = node {
                    Ok(ret.clone())
                } else {
                    Err("Result of LISP must be String".to_string())
                }
            }
            Err(err) => Err(err),
        };
    }

    fn parse(sexp: &String) -> Result<Node, String> {
        let mut tokens = Self::tokenize(sexp);
        let result = Self::_read_from(&mut tokens, 0);
        return result;
    }

    fn eval(node: &Node) -> Result<Node, String> {
        return match node {
            Node::SymbolNode(symbol) => {
                if symbol == "." {
                    Ok(Node::FunctionNode(builtin_string_concat))
                } else if symbol == "current-datetime" {
                    Ok(Node::FunctionNode(builtin_current_datetime))
                } else if symbol == "strftime" {
                    Ok(Node::FunctionNode(builtin_strftime))
                } else {
                    Err("Unknown function: ".to_string() + symbol)
                }
            }
            Node::ListNode(list) => {
                let mut exps: VecDeque<Node> = VecDeque::new();
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
                    return Err("Empty list.".to_string())
                };
                if let Node::FunctionNode(proc) = proc {
                    proc(exps)
                } else {
                    Err("Expected function... But it's not.".to_string())
                }
            }
            _ => Ok(node.clone()),
        };
    }

    fn tokenize(buf: &String) -> VecDeque<String> {
        // TODO This method should care the string literal that contains space character.
        let buf = buf.replace("(", " ( ");
        let buf = buf.replace(")", " ) ");
        let tokens: Vec<&str> = buf.split(" ").collect();
        return tokens
            .iter()
            .filter(|t| !t.is_empty())
            .map(|f| f.to_string())
            .collect();
    }

    fn _read_from(tokens: &mut VecDeque<String>, depth: i32) -> Result<Node, String> {
        if tokens.len() == 0 {
            return Err("Unexpected EOF while reading(LISP)".to_string());
        }

        let Some(token) = tokens.pop_front() else {
            return Err("Missing token... Unexpected EOS.".to_string());
        };
        return if token == "(" {
            let mut values: Vec<Node> = Vec::new();
            while tokens[0] != ")" {
                let result = Self::_read_from(tokens, depth + 1);
                match result {
                    Ok(node) => values.push(node),
                    Err(err) => return Err(err),
                }
            }
            tokens.pop_front();
            Ok(Node::ListNode(values))
        } else if token == ")" {
            Err("Unexpected token: ')'".to_string())
        } else {
            Ok(Self::_atom(&token))
        };
    }

    fn _atom(token: &String) -> Node {
        return if token.len() > 0 && token.starts_with("\"") {
            Node::StringNode(
                token
                    .strip_prefix("\"")
                    .unwrap()
                    .strip_suffix("\"")
                    .unwrap()
                    .to_string()
                    .clone(),
            )
        } else {
            Node::SymbolNode(token.clone())
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
            assert_eq!(node, Node::SymbolNode("hogehoge".to_string()));
        }

        // string node
        {
            let node = TinyLisp::_atom(&"\"hogehoge\"".to_string());
            assert_eq!(node, Node::StringNode("hogehoge".to_string()));
        }
    }

    #[test]
    fn test_tokenize() {
        let tokens = TinyLisp::tokenize(&"(. \"a\" \"b\")".to_string());
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], "(".to_string());
        assert_eq!(tokens[1], ".".to_string());
        assert_eq!(tokens[2], "\"a\"".to_string());
        assert_eq!(tokens[3], "\"b\"".to_string());
        assert_eq!(tokens[4], ")".to_string());
    }

    #[test]
    fn test_run() {
        let p = TinyLisp::run(&"\"hoge\"".to_string()).unwrap();
        assert_eq!(p, "hoge");
    }

    #[test]
    fn test_builtin_concat() {
        let p = TinyLisp::run(&"(. \"h\" \"b\")".to_string()).unwrap();
        assert_eq!(p, "hb");
    }

    #[test]
    fn test_builtin_strftime() {
        let src = "(strftime (current-datetime) \"%Y-%m-%d\")".to_string();
        let parsed = TinyLisp::parse(&src).unwrap();
        dump_node(&parsed, 0);
        let p = TinyLisp::run(&src).unwrap();
        assert_eq!(p.starts_with("2"), true); // this test succeeds until year of 2999.
    }
}
