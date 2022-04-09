use crate::inner_token::Pos;
use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub struct TErr {
    pub pos: Pos,
    pub exp: String,
    pub got: Option<char>,
}

impl std::error::Error for TErr {}
impl Display for TErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let got = match self.got {
            Some(c) => c.to_string(),
            None => "EOI".to_string(),
        };
        write!(
            f,
            "Expected '{}'  but got {} at ({},{})",
            self.exp, got, self.pos.line, self.pos.col
        )
    }
}
