use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum TKind {
    GreaterEqual,
    Ident(String),
    Num(String),
}

struct TestTok<'a> {
    tk: InnerTokenizer<'a>,
}

fn num_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

impl<'a> TestTok<'a> {
    pub fn new(s: &'a str) -> Self {
        TestTok {
            tk: InnerTokenizer::new(s),
        }
    }
    pub fn next(&mut self) -> TokenRes<'a, TKind> {
        self.tk.white_space();
        self.tk.start_token();
        match self.tk.peek_char() {
            Some(c) if num_digit(c) => self.tk.take_while(num_digit, |s| TKind::Num(s.to_string())),
            Some('>') => self.tk.follow('=', TKind::GreaterEqual),
            Some(c) if char::is_alphabetic(c) => self
                .tk
                .take_while(char::is_alphabetic, |s| TKind::Ident(s.to_string())),

            _ => self.tk.expected("A legal token".to_string()),
        }
    }
}

#[test]
fn test_thing_happens() {
    let s = "a >= 54";
    let mut tt = TestTok::new(s);
    assert_eq!(
        tt.next().unwrap().unwrap().value,
        TKind::Ident("a".to_string())
    );
    assert_eq!(tt.next().unwrap().unwrap().value, TKind::GreaterEqual,);
    assert_eq!(
        tt.next().unwrap().unwrap().value,
        TKind::Num("54".to_string())
    );
}
