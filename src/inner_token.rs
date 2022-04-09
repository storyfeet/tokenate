use crate::charbool::CharBool;

use crate::err::TErr;

pub type TokenRes<'a, T> = Result<Option<Token<'a, T>>, TErr>;

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Pos {
    pub i: usize,
    pub line: usize,
    pub col: usize,
}

impl Pos {
    pub fn new() -> Self {
        Self::at(0, 1, 0)
    }
    pub fn at(i: usize, line: usize, col: usize) -> Self {
        Self { i, line, col }
    }
    fn step(&mut self, c: char, i: usize) {
        self.i = i;
        match c {
            '\n' => {
                self.line += 1;
                self.col = 0;
            }
            _ => {
                self.col += 1;
            }
        }
        println!("STEP: {},{} => {:?}", c, i, self);
    }

    fn stepped(mut self, c: char, i: usize) -> Self {
        self.step(c, i);
        self
    }
}

#[derive(Clone, Debug)]
pub struct Token<'a, T> {
    pub value: T,
    pub s: &'a str,
    pub start: Pos,
    pub end: Pos,
}
pub struct InnerTokenizer<'a> {
    s: &'a str,
    pos: Pos,
    chars: std::str::CharIndices<'a>,
    ///The position of the start of the current token
    token_start: Pos,
    peek: Option<(usize, char)>,
}

impl<'a> InnerTokenizer<'a> {
    ///Create a new InnerTokenizer
    pub fn new(s: &'a str) -> Self {
        Self {
            s: s,
            pos: Pos::new(), //starts at 1 because that seems to be how text editors think
            chars: s.char_indices(),
            token_start: Pos::new(),
            peek: None,
        }
    }

    ///Get the next char and it's index
    pub fn next(&mut self) -> Option<(usize, char)> {
        match self.peek.take() {
            Some((n, c)) => {
                self.pos.step(c, n);
                Some((n, c))
            }
            None => {
                let (n, c) = self.chars.next()?;
                println!("NONE NEXT Step '{}',{}", c, n);
                self.pos.step(c, n);
                Some((n, c))
            }
        }
    }

    ///Peek at the next char and index without moving forward
    pub fn peek(&mut self) -> Option<(usize, char)> {
        match &self.peek {
            Some((pi, pc)) => return Some((*pi, *pc)),
            None => {}
        }
        self.peek = self.chars.next();
        self.peek.clone()
    }
    ///Peek, but you don't care about the index
    pub fn peek_char(&mut self) -> Option<char> {
        self.peek().map(|(_, c)| c)
    }
    ///Peek but you only need the index.
    pub fn peek_index(&mut self) -> usize {
        match self.peek() {
            None => self.s.len(),
            Some((i, _)) => i,
        }
    }

    pub fn peek_pos(&mut self) -> Pos {
        match self.peek() {
            None => self.pos.stepped(0 as char, self.s.len()),
            Some((n, c)) => self.pos.stepped(c, n),
        }
    }

    ///Call before creating a token, marks the start before reading it
    ///(token_res calls this ready for the next token, but it is safe to do again)
    pub fn start_token(&mut self) {
        self.token_start = self.peek_pos();
    }

    /// Drop the current peek and make sure new lines and stuff are counted
    /// This guarantees the following call to next (or peek) will be new
    pub fn unpeek(&mut self) {
        match self.peek {
            Some((n, c)) => self.pos.step(c, n),
            _ => {}
        }
        self.peek = None;
    }

    ///Build a TokenRes from the status of the with the string of the current value
    ///unpeek should be true if the last peeked character is part if the current token
    pub fn token_res<T>(&mut self, tt: T, unpeek: bool) -> TokenRes<'a, T> {
        if unpeek {
            self.unpeek();
        }
        Ok(Some(self.make_token(tt)))
    }

    pub fn make_token<T>(&mut self, value: T) -> Token<'a, T> {
        let start = self.token_start;
        let end = self.peek_pos();
        println!("Token Made {:?}", end);
        self.token_start = end;
        Token {
            start,
            end,
            s: &self.s[start.i..end.i],
            value,
        }
    }

    pub fn white_space(&mut self) {
        loop {
            match self.peek_char() {
                Some(c) if c.is_whitespace() => self.unpeek(),
                _ => return,
            }
        }
    }

    pub fn expected<T>(&mut self, s: String) -> TokenRes<'a, T> {
        Err(TErr {
            pos: self.peek_pos(),
            exp: s,
            got: self.peek_char(),
        })
    }

    /// When an item must be followed by a specific character to give a fixed result
    pub fn follow<T, C: CharBool>(&mut self, c: C, t: T) -> TokenRes<'a, T> {
        self.unpeek(); //current peek
        match self.next() {
            Some((_, r)) if c.cb(r) => self.token_res(t, true),
            _ => self.expected(c.expects()),
        }
    }

    /// When an item must be followed by a set of options which could produce different results
    /// ```ignore
    /// // after an equals
    /// tok.follow_fn(|c|match c {
    ///     '>'=>Ok(MyToken::Arrow),
    ///     '='=>Ok(MyToken::EqEq),
    ///     _=>Err("Equals Needs a GT or another Equals"),
    /// })
    /// ```
    pub fn follow_fn<T, F: Fn(char) -> Result<T, String>>(&mut self, f: F) -> TokenRes<'a, T> {
        self.unpeek();
        match self.next() {
            Some((_, r)) => match f(r) {
                Ok(t) => self.token_res(t, true),
                Err(e) => self.expected(e),
            },
            None => self.expected("Not EOI".to_string()),
        }
    }

    /// When an item may be followed by a specific character
    pub fn follow_fn_or<T, F: Fn(char) -> Option<T>>(&mut self, f: F, def: T) -> TokenRes<'a, T> {
        self.unpeek();
        match self.next() {
            Some((_, r)) => match f(r) {
                Some(t) => self.token_res(t, true),
                None => self.token_res(def, false),
            },
            None => self.token_res(def, false),
        }
    }

    pub fn take_while<T, CB: CharBool, F: Fn(&str) -> T>(
        &mut self,
        cb: CB,
        f: F,
    ) -> TokenRes<'a, T> {
        let start = self.peek_index();
        while let Some((end, c)) = self.peek() {
            if !cb.cb(c) {
                let tk = f(&self.s[start..end]);
                return self.token_res(tk, false);
            }
            self.unpeek();
        }
        let tk = f(&self.s[start..]);
        return self.token_res(tk, false);
    }
}
