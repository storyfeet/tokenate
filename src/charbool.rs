pub trait CharBool {
    fn cb(&self, _: char) -> bool;
    fn expects(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
}

impl<F: Fn(char) -> bool> CharBool for F {
    fn cb(&self, c: char) -> bool {
        self(c)
    }
}

impl CharBool for &str {
    fn cb(&self, c: char) -> bool {
        self.contains(c)
    }
    fn expects(&self) -> String {
        format!("One of '{}'", self)
    }
}

impl CharBool for char {
    fn cb(&self, c: char) -> bool {
        *self == c
    }
    fn expects(&self) -> String {
        format!("Char '{}'", self)
    }
}
