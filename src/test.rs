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
    let nx = tt.next().unwrap().unwrap();
    assert_eq!(nx.value, TKind::GreaterEqual,);
    assert_eq!(nx.start, Pos::at(2, 1, 3));
    assert_eq!(nx.end, Pos::at(4, 1, 5));
    assert_eq!(
        tt.next().unwrap().unwrap().value,
        TKind::Num("54".to_string())
    );
}

#[test]
fn test_new_lines_positions() {
    let s = "hit\nbet dog";
    let mut tt = TestTok::new(s);
    //hit
    let nx = tt.next().unwrap().unwrap();
    assert_eq!(nx.start, Pos::at(0, 1, 1));
    assert_eq!(nx.end, Pos::at(3, 2, 0)); //newline starts at \n
    let nx = tt.next().unwrap().unwrap();
    assert_eq!(nx.start, Pos::at(4, 2, 1));
    assert_eq!(nx.end, Pos::at(7, 2, 4));
    let nx = tt.next().unwrap().unwrap();
    assert_eq!(nx.start, Pos::at(8, 2, 5));
    assert_eq!(nx.end, Pos::at(11, 2, 8));
}
