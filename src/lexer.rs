use std::io::Read;
use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub enum Token {
  Eof,
  Def,
  LeftParen,
  RightParen,
  Comma,
  Semi,
  Add,
  Sub,
  Mul,
  Less,
  Extern,
  Identifier(String),
  Number(f64),
}

pub struct Lexer {
  peeker: Peekable<Box<dyn Iterator<Item = u8>>>,
  tok_1st: Token,
  tok_2nd: Token,
}

impl Lexer {
  pub fn new(reader: impl Read + 'static) -> Lexer {
    let bytes: Box<dyn Iterator<Item = u8>> = Box::new(reader.bytes().filter_map(Result::ok));
    let mut lexer = Self {
      peeker: bytes.peekable(),
      tok_1st: Token::Eof,
      tok_2nd: Token::Eof,
    };
    lexer.tok_1st = lexer.get_tok();
    lexer.tok_2nd = lexer.get_tok();
    lexer
  }

  pub fn peek_first(&self) -> &Token {
    &self.tok_1st
  }

  pub fn peek_second(&self) -> &Token {
    &self.tok_2nd
  }

  pub fn next_token(&mut self) -> Token {
    let tmp = self.tok_2nd.clone();
    self.tok_2nd = self.get_tok();
    let res = self.tok_1st.clone();
    self.tok_1st = tmp;
    res
  }

  fn get_tok(&mut self) -> Token {
    let peeked = self.peeker.next();
    match peeked {
      None => Token::Eof,
      Some(b'(') => Token::LeftParen,
      Some(b')') => Token::RightParen,
      Some(b',') => Token::Comma,
      Some(b';') => Token::Semi,
      Some(b'+') => Token::Add,
      Some(b'-') => Token::Sub,
      Some(b'*') => Token::Mul,
      Some(b'<') => Token::Less,
      Some(b'#') => {
        while let Some(_) = self.peeker.next_if(|x| *x != b'\n') {}
        self.peeker.next();
        self.get_tok()
      }
      Some(c) if c.is_ascii_whitespace() => {
        while let Some(_) = self.peeker.next_if(u8::is_ascii_whitespace) {}
        self.get_tok()
      }
      Some(c) if c.is_ascii_alphabetic() => {
        let mut ident = vec![c];
        while let Some(x) = self.peeker.next_if(u8::is_ascii_alphanumeric) {
          ident.push(x);
        }
        let ident = String::from_utf8(ident).unwrap();
        match ident.as_str() {
          "def" => Token::Def,
          "extern" => Token::Extern,
          _ => Token::Identifier(ident),
        }
      }
      Some(c) if c.is_ascii_digit() => {
        let mut num = vec![c];
        while let Some(x) = self.peeker.next_if(|x| x.is_ascii_digit() || *x == b'.') {
          num.push(x);
        }
        let num: f64 = String::from_utf8(num).unwrap().parse().unwrap();
        Token::Number(num)
      }
      _ => Token::Def,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Cursor;

  #[test]
  fn token_eof() {
    let source = b"";
    let reader = Cursor::new(&source[..]);
    let mut lexer = Lexer::new(reader);
    assert_eq!(lexer.next_token(), Token::Eof);
  }

  #[test]
  fn token_parenthese_comma() {
    let source = b"(,)";
    let mut lexer = Lexer::new(Cursor::new(&source[..]));
    assert_eq!(lexer.next_token(), Token::LeftParen);
    assert_eq!(lexer.next_token(), Token::Comma);
    assert_eq!(lexer.next_token(), Token::RightParen);
  }

  #[test]
  fn token_numbers() {
    let source = "3.14";
    let mut lexer = Lexer::new(Cursor::new(&source[..]));
    assert_eq!(lexer.next_token(), Token::Number(3.14_f64));
    assert_eq!(lexer.next_token(), Token::Eof);
  }

  #[test]
  fn token_identifiers() {
    let source = "foo def bar extern";
    let mut lexer = Lexer::new(Cursor::new(&source[..]));
    assert_eq!(lexer.next_token(), Token::Identifier("foo".to_string()));
    assert_eq!(lexer.next_token(), Token::Def);
    assert_eq!(lexer.next_token(), Token::Identifier("bar".to_string()));
    assert_eq!(lexer.next_token(), Token::Extern);
    assert_eq!(lexer.next_token(), Token::Eof);
  }

  #[test]
  fn token_comment() {
    let source = "def foo  # this is commment \n 42";
    let mut lexer = Lexer::new(Cursor::new(&source[..]));
    assert_eq!(lexer.next_token(), Token::Def);
    assert_eq!(lexer.next_token(), Token::Identifier("foo".to_string()));
    assert_eq!(lexer.next_token(), Token::Number(42.0_f64));
    assert_eq!(lexer.next_token(), Token::Eof);
  }
}
