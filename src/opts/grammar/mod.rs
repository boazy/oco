use enum_dispatch::enum_dispatch;

pub use crate::opts::grammar::gnu::Gnu;
use crate::opts::parsed_args::ParsedArgs;
use crate::cli::Dialect;
use eyre::Result;

mod gnu;

#[enum_dispatch]
pub trait Grammar : Clone {
    fn parse<I: Iterator<Item = String>>(&self, args: I) -> Result<ParsedArgs>;

    fn generate<F : FnMut(String)>(&self, args: ParsedArgs, f: F) -> Result<()>;

    fn generate_vec(&self, args: ParsedArgs) -> Result<Vec<String>> {
        let mut vec = Vec::new();
        self.generate(args, |s| vec.push(s))?;
        Ok(vec)
    }
}


