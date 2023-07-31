#![allow(unused)]
use crate::lexer::{Lexer, Token};

#[derive(Debug, PartialEq)]
pub enum Ast {
  Expr(ExprAst),
  Proto(ProtoAst),
  Func(FuncAst),
}

#[derive(Debug, PartialEq)]
pub enum ExprAst {
  NumAst(f64),
  VarAst(String),
  BinAst(Box<ExprAst>, char, Box<ExprAst>),
  CallAst(String, Vec<ExprAst>),
}

#[derive(Debug, PartialEq)]
pub struct ProtoAst {
  name: String,
  args: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct FuncAst {
  proto: ProtoAst,
  body: ExprAst,
}

impl Ast {
  // pub fn new(lexer: Lexer) -> Self {}
  pub fn parse(lexer: &mut Lexer) -> Self {
    match lexer.peek_first() {
      &Token::Extern => Self::parse_extern(lexer),
      &Token::Def => Self::Func(FuncAst::parse(lexer)),
      _ => Self::parse_top_level_expr(lexer),
    }
  }

  fn parse_extern(lexer: &mut Lexer) -> Self {
    lexer.next_token(); // eat `extern`
    Self::Proto(ProtoAst::parse(lexer))
  }

  fn parse_top_level_expr(lexer: &mut Lexer) -> Self {
    let expr = ExprAst::parse(lexer);
    let proto = ProtoAst {
      name: String::new(),
      args: vec![],
    };
    Self::Func(FuncAst { proto, body: expr })
  }
}

impl ExprAst {
  fn parse(lexer: &mut Lexer) -> Self {
    let lhs = Self::parse_primary(lexer);
    Self::parse_bin_rhs(lexer, lhs, 0)
  }

  fn parse_bin_rhs(lexer: &mut Lexer, lhs: ExprAst, prec_prev: i8) -> Self {
    let prec_cur = Self::get_precedence(lexer.peek_first());
    if prec_cur <= prec_prev {
      return lhs;
    }

    let operator = match lexer.next_token() {
      Token::Less => '<',
      Token::Add => '+',
      Token::Sub => '-',
      Token::Mul => '*',
      _ => panic!(),
    };
    let mut rhs = Self::parse_primary(lexer);
    let mut prec_next = Self::get_precedence(lexer.peek_first());

    loop {
      if prec_next <= prec_cur {
        let lhs_new = Self::BinAst(Box::new(lhs), operator, Box::new(rhs));
        break Self::parse_bin_rhs(lexer, lhs_new, prec_prev);
      } else {
        rhs = Self::parse_bin_rhs(lexer, rhs, prec_cur);
        prec_next = Self::get_precedence(lexer.peek_first());
      }
    }
  }

  fn parse_primary(lexer: &mut Lexer) -> Self {
    match lexer.peek_first() {
      &Token::Number(_) => Self::parse_number(lexer),
      &Token::LeftParen => Self::parse_paren(lexer),
      &Token::Identifier(_) => match lexer.peek_second() {
        &Token::LeftParen => Self::parse_call(lexer),
        _ => Self::parse_var(lexer),
      },
      _ => panic!(),
    }
  }

  fn parse_number(lexer: &mut Lexer) -> Self {
    let Token::Number(n) = lexer.next_token() else {panic!()};
    Self::NumAst(n)
  }

  fn parse_paren(lexer: &mut Lexer) -> Self {
    lexer.next_token(); // eat `(`
    let expr = Self::parse(lexer);

    match lexer.peek_first() {
      &Token::RightParen => {
        lexer.next_token();
      } // eat `)`
      _ => panic!("Expected `)` token"),
    }
    expr
  }

  fn parse_var(lexer: &mut Lexer) -> Self {
    let Token::Identifier(s) = lexer.next_token() else {panic!("Expected Identifier token")};
    Self::VarAst(s)
  }

  fn parse_call(lexer: &mut Lexer) -> Self {
    let Token::Identifier(name) = lexer.next_token() else {panic!("Expected Identifier token")};
    lexer.next_token(); // eat `(`
    let mut args = vec![];
    loop {
      if lexer.peek_first() == &Token::RightParen {
        break;
      }
      args.push(Self::parse(lexer));
      match lexer.peek_first() {
        &Token::RightParen => break,
        &Token::Comma => {
          lexer.next_token();
        }
        _ => panic!("Expected ')' or ',' in argument list"),
      }
    }
    lexer.next_token(); // eat `)`
    Self::CallAst(name, args)
  }

  fn get_precedence(token: &Token) -> i8 {
    match token {
      &Token::Less => 10,
      &Token::Add => 20,
      &Token::Sub => 20,
      &Token::Mul => 40,
      _ => -1, // other tokens means the ending of a binary expression
    }
  }
}

impl ProtoAst {
  fn parse(lexer: &mut Lexer) -> Self {
    let Token::Identifier(name) = lexer.next_token() else {panic!("Expect an identifier")};
    lexer.next_token(); // eat `(`
    let mut args = vec![];
    loop {
      match lexer.next_token() {
        Token::RightParen => break,
        Token::Comma => (),
        Token::Identifier(s) => args.push(s),
        _ => panic!(),
      }
    }
    Self { name, args }
  }
}

impl FuncAst {
  fn parse(lexer: &mut Lexer) -> Self {
    lexer.next_token(); // eat `def`
    let proto = ProtoAst::parse(lexer);
    let body = ExprAst::parse(lexer);
    Self { proto, body }
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer;

  use super::*;
  use std::io::Cursor;

  #[test]
  fn expr_number() {
    let src = " 42 ";
    let mut lexer = Lexer::new(Cursor::new(src));
    let ast = ExprAst::parse(&mut lexer);
    assert_eq!(ast, ExprAst::NumAst(42.0));
  }

  #[test]
  fn expr_variable() {
    let src = "foo";
    let mut lexer = Lexer::new(Cursor::new(src));
    let ast = ExprAst::parse(&mut lexer);
    assert_eq!(ast, ExprAst::VarAst("foo".to_string()));
  }

  #[test]
  fn expr_paren() {
    let src = "(foo )";
    let mut lexer = Lexer::new(Cursor::new(src));
    let ast = ExprAst::parse(&mut lexer);
    assert_eq!(ast, ExprAst::VarAst("foo".to_string()));
  }

  #[test]
  fn expr_bin_expr_1() {
    let src = "1 + foo";
    let mut lexer = Lexer::new(Cursor::new(src));
    let ast = ExprAst::parse(&mut lexer);
    assert_eq!(
      ast,
      ExprAst::BinAst(
        Box::new(ExprAst::NumAst(1.0)),
        '+',
        Box::new(ExprAst::VarAst("foo".to_string()))
      )
    );
  }

  #[test]
  fn expr_bin_expr_2() {
    let src = "1 + foo * 42";
    let mut lexer = Lexer::new(Cursor::new(src));
    let ast = ExprAst::parse(&mut lexer);
    assert_eq!(
      ast,
      ExprAst::BinAst(
        Box::new(ExprAst::NumAst(1.0)),
        '+',
        Box::new(ExprAst::BinAst(
          Box::new(ExprAst::VarAst("foo".to_string())),
          '*',
          Box::new(ExprAst::NumAst(42.0)),
        ))
      )
    )
  }

  #[test]
  fn expr_bin_expr_3() {
    let src = "1 + foo - 42";
    let mut lexer = Lexer::new(Cursor::new(src));
    let ast = ExprAst::parse(&mut lexer);
    assert_eq!(
      ast,
      ExprAst::BinAst(
        Box::new(ExprAst::BinAst(
          Box::new(ExprAst::NumAst(1.0)),
          '+',
          Box::new(ExprAst::VarAst("foo".to_string())),
        )),
        '-',
        Box::new(ExprAst::NumAst(42.0)),
      )
    )
  }

  #[test]
  fn expr_bin_expr_4() {
    use ExprAst::*;
    let src = "1 < foo + bar * 42 - baz";
    let mut lexer = Lexer::new(Cursor::new(src));
    let ast = ExprAst::parse(&mut lexer);
    assert_eq!(
      ast,
      BinAst(
        Box::new(NumAst(1.0)),
        '<',
        Box::new(BinAst(
          Box::new(BinAst(
            Box::new(VarAst("foo".to_string())),
            '+',
            Box::new(BinAst(
              Box::new(VarAst("bar".to_string())),
              '*',
              Box::new(NumAst(42.0))
            )),
          )),
          '-',
          Box::new(VarAst("baz".to_string())),
        )),
      )
    );
  }

  #[test]
  fn expr_func_call() {
    let src = "foo(1 + 2, bar, 42)";
    let mut lexer = Lexer::new(Cursor::new(src));
    let ast = ExprAst::parse(&mut lexer);
    use ExprAst::*;
    assert_eq!(
      ast,
      CallAst(
        "foo".to_string(),
        vec![
          BinAst(Box::new(NumAst(1.0)), '+', Box::new(NumAst(2.0))),
          VarAst("bar".to_string()),
          NumAst(42.0),
        ]
      )
    )
  }

  #[test]
  fn proto() {
    let src = "foo(a, b, c);";
    let mut lexer = Lexer::new(Cursor::new(src));
    let ast = ProtoAst::parse(&mut lexer);
    assert_eq!(
      ast,
      ProtoAst {
        name: "foo".to_string(),
        args: vec!["a".to_string(), "b".to_string(), "c".to_string()],
      }
    )
  }

  #[test]
  fn parse_function() {
    let src = "def foo(a, b, c) a+b*c";
    let mut lexer = Lexer::new(Cursor::new(src));
    let ast = FuncAst::parse(&mut lexer);
    use ExprAst::*;
    assert_eq!(
      ast,
      FuncAst {
        proto: ProtoAst {
          name: "foo".to_string(),
          args: vec!["a".to_string(), "b".to_string(), "c".to_string()]
        },
        body: BinAst(
          Box::new(VarAst("a".to_string())),
          '+',
          Box::new(BinAst(
            Box::new(VarAst("b".to_string())),
            '*',
            Box::new(VarAst("c".to_string()))
          ))
        )
      }
    )
  }
}
