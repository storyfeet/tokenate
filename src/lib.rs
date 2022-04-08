mod charbool;
pub use charbool::CharBool;
pub struct TErr {
    line: usize,
    cnum: usize,
    index: usize,
    exp: &'static str,
    got: Option<char>,
}

pub type TokenRes<'a, T> = Result<Option<Token<'a, T>>, TErr>;

pub struct Token<'a, T> {
    value: T,
    s: &'a str,
    start: usize,
    end: usize,
}
pub struct TokenChars<'a> {
    s: &'a str,
    line: usize,
    chars: std::str::CharIndices<'a>,
    start: usize,
    peek: Option<(usize, char)>,
}

impl<'a> TokenChars<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            s: s,
            line: 1, //starts at 1 because that seems to be how text editors think
            chars: s.char_indices(),
            start: 0,
            peek: None,
        }
    }

    pub fn next_char(&mut self) -> Option<(usize, char)> {
        match self.peek.take() {
            Some((n, c)) if c == '\n' => {
                self.line += 1;
                Some((n, c))
            }
            Some(c) => Some(c),
            None => self.chars.next(),
        }
    }

    pub fn peek_char(&mut self) -> Option<(usize, char)> {
        let (ni, nc) = self.next_char()?;
        self.peek = Some((ni, nc));
        Some((ni, nc))
    }

    pub fn unpeek(&mut self) {
        match self.peek {
            Some((_, c)) if c == 'n' => self.line += 1,
            _ => {}
        }
        self.peek = None;
    }

    pub fn peek_index(&mut self) -> usize {
        match self.peek_char() {
            None => self.s.len(),
            Some((i, _)) => i,
        }
    }

    pub fn make_token_res<T>(&mut self, tt: T) -> TokenRes<'a, T> {
        Ok(Some(self.make_token(tt)))
    }

    pub fn make_token<T>(&mut self, value: T) -> Token<'a, T> {
        self.unpeek();
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
                Some((_, c)) if c.is_whitespace() => {
                    self.peek = None;
                }
                _ => return,
            }
        }
    }


    fn expected<T>(&mut self, s: &str) -> TokenRes<'a, T> {
        let (index,cnum )match self.peek_ne
        TErr{
            line: self.num,
            index: 
            exp: &'static str,
            got: Option<char>,
        }
    }

    pub fn require<T, C: CharBool>(&mut self, c: C, t: T) -> TokenRes<'a, T> {
        self.unpeek(); //current peek
        match self.next_char() {
            Some((_, r)) if c.cb(r) => self.make_token_res(t),
            _ => self.expected(c.expects()),
        }
    }

    /*pub fn next(&mut self) -> TokenRes<'a> {
    let follow = |s: &mut Self, c: char, tt: TokenType<'a>| {
        s.peek = None;
        match s.next_char() {
            Some((_, r)) if r == c => s.make_token_wrap(tt, false),
            _ => e_str("Expected second Dot"),
        }
    };
    let follow_def = |s: &mut Self, c: char, tt: TokenType<'a>, def: TokenType<'a>| {
        s.peek = None;
        match s.peek_char() {
            Some((_, r)) if r == c => s.make_token_wrap(tt, true),
            _ => s.make_token_wrap(def, false),
        }
    };
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
