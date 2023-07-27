#![allow(unused)]
use crate::lexer::{Lexer, Token};

/// ExprAST - represents all expression nodes.
enum ExprAST {
  NumberExpr(f64),
  VariableExpr(String),
  BinaryExpr(Box<ExprAST>, char, Box<ExprAST>), // LHS, op, RHS
  CallExpr(String, Vec<ExprAST>),               // func-name, args
}

impl ExprAST {
  fn parse(lexer: &mut Lexer) -> Self {
    let lhs = Self::parse_primary(lexer);
    Self::parse_bin_ops(lexer, 0, lhs)
  }

  fn parse_bin_ops(lexer: &mut Lexer, expr_prec: i8, lhs: ExprAST) -> Self {
    let tok_prec = precedence(lexer.peek_first());

    // If this is a binop that binds at least as tightly as
    // the current binop, consume it, otherwise we are done.
    if tok_prec < expr_prec {
      return lhs;
    }

    // Okay, we know this is a binop
    let binop = match lexer.next_token() {
      Token::Add => '+',
      Token::Sub => '-',
      Token::Less => '<',
      Token::Mul => '*',
      _ => panic!(),
    };

    // parse the primary expression after the binary operator
    let mut rhs = Self::parse_primary(lexer);

    // If BinOp binds less tightly with RHS than the operator
    // after RHS, let the pending operator take RHS as it LHS.
    let next_prec = precedence(lexer.peek_first());
    if tok_prec < next_prec {
      rhs = Self::parse_bin_ops(lexer, tok_prec + 1, rhs);
    }

    // merge LHS/RHS
    let lhs = Self::BinaryExpr(Box::new(lhs), binop, Box::new(rhs));
    Self::parse_bin_ops(lexer, expr_prec, lhs)
  }

  fn parse_primary(lexer: &mut Lexer) -> Self {
    let token = lexer.peek_first();
    match *token {
      Token::Number(n) => {
        lexer.next_token();
        Self::NumberExpr(n)
      }
      Token::LeftParen => Self::parse_paren(lexer),
      Token::Identifier(_) => {
        if lexer.peek_second() == &Token::LeftParen {
          Self::parse_call(lexer)
        } else {
          let Token::Identifier(s) = lexer.next_token() else {panic!()};
          Self::VariableExpr(s)
        }
      }
      _ => panic!(),
    }
  }

  fn parse_paren(lexer: &mut Lexer) -> Self {
    lexer.next_token(); // eat `(`
    let val = Self::parse(lexer);
    if lexer.peek_first() != &Token::RightParen {
      panic!()
    }
    lexer.next_token(); // eat `)`
    val
  }

  fn parse_call(lexer: &mut Lexer) -> Self {
    let Token::Identifier(name) = lexer.next_token() else {panic!()};
    lexer.next_token(); // eat `(`
    let mut args = vec![];
    loop {
      args.push(Self::parse(lexer));
      if lexer.peek_first() == &Token::RightParen {
        lexer.next_token(); // eat `)`
        break;
      }
      if lexer.peek_first() == &Token::Comma {
        lexer.next_token();
      } else {
        panic!();
      }
    }
    Self::CallExpr(name, args)
  }
}

fn precedence(tok: &Token) -> i8 {
  match tok {
    &Token::Less => 10,
    &Token::Add => 20,
    &Token::Sub => 20,
    &Token::Mul => 40,
    _ => -1,
  }
}

/// PrototypeAST - represents the "prototype" for a function,
/// which captures its name, and its argument names (thus implicitly the
/// number of arguments the function takes).
struct PrototypeAST {
  name: String,
  args: Vec<String>,
}

impl PrototypeAST {
  fn parse(lexer: &mut Lexer) -> Self {
    let Token::Identifier(name) = lexer.next_token() else { panic!() };
    let tok = lexer.next_token();
    assert_eq!(tok, Token::LeftParen);

    let mut args = vec![];
    while let Token::Identifier(s) = lexer.next_token() {
      args.push(s);
    }
    if lexer.peek_first() != &Token::RightParen {
      panic!();
    }
    lexer.next_token();
    Self { name, args }
  }
}

/// FunctionAST - represents a function definition itself.
struct FunctionAST {
  proto: PrototypeAST,
  body: ExprAST,
}

impl FunctionAST {
  fn parse(lexer: &mut Lexer) -> Self {
    lexer.next_token(); // eat `def`
    let proto = PrototypeAST::parse(lexer);
    let body = ExprAST::parse(lexer);
    Self { proto, body }
  }
}

enum Ast {
  ExprAST(ExprAST),
  PrototypeAST(PrototypeAST),
  FunctionAST(FunctionAST),
}

struct Parser {
  buf: Vec<Ast>,
}

impl Parser {
  pub fn new() -> Self {
    Self { buf: vec![] }
  }

  pub fn parse_ast(&mut self, lexer: &mut Lexer) {
    loop {
      let token = lexer.peek_first();
      match token {
        &Token::Eof => break,
        &Token::Semi => {
          lexer.next_token();
        }
        &Token::Def => {
          self.buf.push(Ast::FunctionAST(FunctionAST::parse(lexer)));
        }
        &Token::Extern => {
          lexer.next_token();
          self.buf.push(Ast::PrototypeAST(PrototypeAST::parse(lexer)));
        }
        _ => {
          self.buf.push(Ast::ExprAST(ExprAST::parse(lexer)));
        }
      }
    }
  }
}
