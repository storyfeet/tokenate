pub struct TErr {
    line: u64,
    cnum: u64,
    index: u64,
    exp: &'static str,
    got: Option<char>,
}

pub type TokenRes<'a, T> = Result<Option<Token<'a, T>>, TErr>;

pub struct Token<'a, T> {
    kind: T,
    s: &'a str,
}
pub struct TokenChars<'a> {
    s: &'a str,
    chars: std::str::CharIndices<'a>,
    start: u64,
    peek: Option<(usize, char)>,
}

impl<'a> TokenChars<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            s: s,
            chars: s.char_indices(),
            start: 0,
            peek: None,
        }
    }

    pub fn next_char(&mut self) -> Option<(usize, char)> {
        match self.peek.take() {
            Some((_,c))if c == '\n' 
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

    pub fn make_token_res<T>(&mut self, tt: T, unpeek: bool) -> TokenRes<'a, T> {
        if unpeek {
            self.peek = None
        }
        Ok(Some(self.make_token(tt)))
    }

    pub fn make_token<T>(&mut self, tt: T) -> Token<'a, T> {
        let start = self.start;
        let end = self.peek_index();
        self.start = end;
        Token {
            start,
            end,
            s: &self.s[start..end],
            tt,
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

    pub fn follow<T, F: Fn(char) -> T>(&mut self) -> TokenRes<'a, T> {}

    pub fn next(&mut self) -> TokenRes<'a> {
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
            '(' => self.make_token_wrap(TokenType::ParenO, true),
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
            c if c.is_alphabetic() || c == '_' => self.unqoth(),

            _ => e_str("Unexpected Character"),
        }
    }
}
