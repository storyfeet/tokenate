Tokenate
=======

A library to make tokenizing a string easier, it is lazy in evaluation, and will only return the next token as needed.

Your start point is the InnerTokenizer which is intended to be a child to your actual tokenizer,
providing the indexing utility, so you can worry about consuming characters.
The main methods you will need are "start_token" "next" "peek" and "unpeek", then finally "token_res"
which will build the resulting Token with Line Number and Value
The other methods like "follow","follow_fn" and "take_while" are helpers to make getting
strings and numbers more easily

The Token::value is generic allowing you to choose define how tokens for your language look.

```rust
use tokenate::*;

// Define the Tokens you will want to recieve
#[derive(Clone, Debug, PartialEq)]
pub enum TKind {
    GreaterEqual,
    Ident(String),
    Num(String),
}

//Crate your tokenizer include the inner
struct TestTok<'a> {
    tk: InnerTokenizer<'a>,
}

//A util method for checking we have a char
fn num_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}


impl<'a> TestTok<'a> {
    // When created initialize the InnerTokenizer
    pub fn new(s: &'a str) -> Self {
        TestTok {
            tk: InnerTokenizer::new(s),
        }
    }


    // TokenRes<T> is Result<Option<Token<T>>,TErr>
    pub fn next(&mut self) -> TokenRes<'a, TKind> {
        //Skip characters we don't care about
        self.tk.skip(char::is_whitespace);
        //Begin the token
        self.tk.start_token();
        // Use peek_char to decide wich kind of token to make
        match self.tk.peek_char() {
            Some(c) if num_digit(c) => self.tk.take_while(num_digit, |s| Ok(TKind::Num(s.to_string()))),
            Some('>') => self.tk.follow('=', TKind::GreaterEqual),
            Some(c) if char::is_alphabetic(c) => self
                .tk
                .take_while(char::is_alphabetic, |s| Ok(TKind::Ident(s.to_string()))),

            _ => self.tk.expected("A legal token".to_string()),
        }
    }
}


//A test function
fn main() {
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
```
