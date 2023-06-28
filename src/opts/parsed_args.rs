#[derive(Debug)]
pub struct ParsedArgs {
    pub options: Vec<ParsedOpt>,
    pub positional: Vec<String>,
}

#[derive(Debug)]
pub struct ParsedOpt {
    pub name: String,
    pub values: Vec<String>,
    pub short: bool,
}

impl ParsedOpt {
    pub fn value(&self) -> Option<&String> {
        self.values.first()
    }
}
