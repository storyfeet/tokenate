//! An InnerTokenizer is intended to be a child to your actual tokenizer, providing the string
//! and indexing utility, so you can worry about consuming characters.
//! The main methods you will need are "start_token" "next" "peek" and "unpeek", then finally "token_res"
//! which will build the resulting Token with Line Number and Value
//! The other methods like "follow","follow_fn" and "take_while" are helpers to make getting
//! strings and numbers more easily
//! The Token::value is generic allowing you to choose define how tokens for your language look.
//!
//! ```rust
//! use tokenate::*;
//!
//! #[derive(Clone, Debug, PartialEq)]
//! pub enum TKind {
//!     GreaterEqual,
//!     Ident(String),
//!     Num(String),
//! }
//!
//! struct TestTok<'a> {
//!     tk: InnerTokenizer<'a>,
//! }
//!
//! fn num_digit(c: char) -> bool {
//!     c >= '0' && c <= '9'
//! }
//!
//! impl<'a> TestTok<'a> {
//!     pub fn new(s: &'a str) -> Self {
//!         TestTok {
//!             tk: InnerTokenizer::new(s),
//!         }
//!     }
//!     pub fn next(&mut self) -> TokenRes<'a, TKind> {
//!         self.tk.white_space();
//!         self.tk.start_token();
//!         match self.tk.peek_char() {
//!             Some(c) if num_digit(c) => self.tk.take_while(num_digit, |s| TKind::Num(s.to_string())),
//!             Some('>') => self.tk.follow('=', TKind::GreaterEqual),
//!             Some(c) if char::is_alphabetic(c) => self
//!                 .tk
//!                 .take_while(char::is_alphabetic, |s| TKind::Ident(s.to_string())),
//!
//!             _ => self.tk.expected("A legal token".to_string()),
//!         }
//!     }
//! }
//!
//!
//! fn main() {
//!     let s = "a >= 54";
//!     let mut tt = TestTok::new(s);
//!     assert_eq!(
//!         tt.next().unwrap().unwrap().value,
//!         TKind::Ident("a".to_string())
//!     );
//!     assert_eq!(tt.next().unwrap().unwrap().value, TKind::GreaterEqual,);
//!     assert_eq!(
//!         tt.next().unwrap().unwrap().value,
//!         TKind::Num("54".to_string())
//!     );
//! }
//! ```

mod charbool;
mod err;
mod inner_token;
#[cfg(test)]
mod test;

pub use charbool::CharBool;
pub use err::TErr;
pub use inner_token::*;
