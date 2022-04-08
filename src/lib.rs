mod charbool;
mod err;
//#[cfg(test)]
mod test;
pub use charbool::CharBool;
pub use err::TErr;

pub type TokenRes<'a, T> = Result<Option<Token<'a, T>>, TErr>;

#[derive(Clone, Debug)]
pub struct Token<'a, T> {
    pub value: T,
    pub s: &'a str,
    pub start: usize,
    pub end: usize,
}

/// An InnerTokenizer is intended to be a child to your actual tokenizer, providing the string
/// and indexing utility, so you can worry about consuming characters.
/// Rather than have to places for storage it runs with a peek that can be viewed, and then called
/// upon later.
/// you can either call next_char or peek_char to get the next character with it's index
pub struct InnerTokenizer<'a> {
    s: &'a str,
    line: usize,
    col: usize,
    chars: std::str::CharIndices<'a>,
    start: usize,
    peek: Option<(usize, char)>,
}

impl<'a> InnerTokenizer<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            s: s,
            line: 1, //starts at 1 because that seems to be how text editors think
            col: 0,
            chars: s.char_indices(),
            start: 0,
            peek: None,
        }
    }

    pub fn next(&mut self) -> Option<(usize, char)> {
        match self.peek.take() {
            Some((n, c)) if c == '\n' => {
                self.line += 1;
                Some((n, c))
            }
            Some(c) => Some(c),
            None => self.chars.next(),
        }
    }
    pub fn peek(&mut self) -> Option<(usize, char)> {
        let (ni, nc) = self.next()?;
        self.peek = Some((ni, nc));
        Some((ni, nc))
    }
    pub fn peek_char(&mut self) -> Option<char> {
        self.peek().map(|(_, c)| c)
    }
    pub fn peek_index(&mut self) -> usize {
        match self.peek() {
            None => self.s.len(),
            Some((i, _)) => i,
        }
    }

    /// Drop the current peek and make sure new lines and stuff are counted
    /// This guarantees the following call to next (or peek) will be new
    pub fn unpeek(&mut self) {
        match self.peek {
            Some((_, c)) if c == 'n' => self.line += 1,
            _ => {}
        }
        self.peek = None;
    }

    pub fn token_res<T>(&mut self, tt: T, unpeek: bool) -> TokenRes<'a, T> {
        if unpeek {
            self.unpeek();
        }
        Ok(Some(self.make_token(tt)))
    }

    pub fn make_token<T>(&mut self, value: T) -> Token<'a, T> {
        let start = self.start;
        let end = self.peek_index();
        self.start = end;
        Token {
            start,
            end,
            s: &self.s[start..end],
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

    fn expected<T>(&mut self, s: String) -> TokenRes<'a, T> {
        let (index, got) = match self.peek() {
            Some((n, c)) => (n, Some(c)),
            None => (self.s.len(), None),
        };
        Err(TErr {
            line: self.line,
            col: self.col,
            index,
            exp: s,
            got,
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

    /*pub fn next(&mut self) -> TokenRes<'a> {
    self.white_space();
    self.start = self.peek_index();
    let pc = match self.peek_char() {
        None => return Ok(None),
        Some(v) => v,
    };
    match pc.1 {
        c if c >= '0' && c <= '9' => self.number(),
        '\"' => self.qoth(),
        /*    '(' => self.make_token_wrap(TokenType::ParenO, true),
        ')' => self.make_token_wrap(TokenType::ParenC, true),
        '[' => self.make_token_wrap(TokenType::BraceO, true),
        ']' => self.make_token_wrap(TokenType::BraceC, true),
        '+' => follow_def(self, '+', TokenType::Append, TokenType::Add),
        '-' => self.make_token_wrap(TokenType::Sub, true),
        '$' => self.make_token_wrap(TokenType::Dollar, true),
        ':' => self.make_token_wrap(TokenType::Colon, true),
        ',' => self.make_token_wrap(TokenType::Comma, true),
        '.' => follow(self, '.', TokenType::Range),
        '=' => follow(self, '=', TokenType::Equal),
        '<' => self.make_token_wrap(TokenType::Less, true),
        '>' => self.make_token_wrap(TokenType::Greater, true),
        '!' => self.make_token_wrap(TokenType::Count, true),
        c if c.is_alphabetic() || c == '_' => self.unqoth(),*/
            _ => self.expected("string"),
        }
    }*/
}
