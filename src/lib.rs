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
/// The main methods you will need are "start_token" "next" "peek" and "unpeek", then finally "token_res"
/// which will build the resulting Token with Line Number and Value
/// The other methods like "follow","follow_fn" and "take_while" are helpers to make getting
/// strings and numbers more easily
/// The Token::value is generic allowing you to choose define how tokens for your language look.
///
/// ```rust
/// use tokenize::*;
///
/// #[derive(Clone, Debug, PartialEq)]
/// pub enum TKind {
///     GreaterEqual,
///     Ident(String),
///     Num(String),
/// }
///
/// struct TestTok<'a> {
///     tk: InnerTokenizer<'a>,
/// }
///
/// fn num_digit(c: char) -> bool {
///     c >= '0' && c <= '9'
/// }
///
/// impl<'a> TestTok<'a> {
///     pub fn new(s: &'a str) -> Self {
///         TestTok {
///             tk: InnerTokenizer::new(s),
///         }
///     }
///     pub fn next(&mut self) -> TokenRes<'a, TKind> {
///         self.tk.white_space();
///         self.tk.start_token();
///         match self.tk.peek_char() {
///             Some(c) if num_digit(c) => self.tk.take_while(num_digit, |s| TKind::Num(s.to_string())),
///             Some('>') => self.tk.follow('=', TKind::GreaterEqual),
///             Some(c) if char::is_alphabetic(c) => self
///                 .tk
///                 .take_while(char::is_alphabetic, |s| TKind::Ident(s.to_string())),
///
///             _ => self.tk.expected("A legal token".to_string()),
///         }
///     }
/// }
///
///
/// fn main() {
///     let s = "a >= 54";
///     let mut tt = TestTok::new(s);
///     assert_eq!(
///         tt.next().unwrap().unwrap().value,
///         TKind::Ident("a".to_string())
///     );
///     assert_eq!(tt.next().unwrap().unwrap().value, TKind::GreaterEqual,);
///     assert_eq!(
///         tt.next().unwrap().unwrap().value,
///         TKind::Num("54".to_string())
///     );
/// }
/// ```
pub struct InnerTokenizer<'a> {
    s: &'a str,
    line: usize,
    col: usize,
    chars: std::str::CharIndices<'a>,
    ///The position of the start
    token_start: usize,
    peek: Option<(usize, char)>,
}

impl<'a> InnerTokenizer<'a> {
    ///Create a new InnerTokenizer
    pub fn new(s: &'a str) -> Self {
        Self {
            s: s,
            line: 1, //starts at 1 because that seems to be how text editors think
            col: 0,
            chars: s.char_indices(),
            token_start: 0,
            peek: None,
        }
    }

    ///Get the next char and it's index
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

    ///Peek at the next char and index without moving forward
    pub fn peek(&mut self) -> Option<(usize, char)> {
        let (ni, nc) = self.next()?;
        self.peek = Some((ni, nc));
        Some((ni, nc))
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

    ///Call before creating a token, marks the start before reading it
    ///(token_res calls this ready for the next token, but it is safe to do again)
    pub fn start_token(&mut self) {
        self.token_start = self.peek_index();
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
        let end = self.peek_index();
        self.token_start = end;
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

    pub fn expected<T>(&mut self, s: String) -> TokenRes<'a, T> {
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
}
