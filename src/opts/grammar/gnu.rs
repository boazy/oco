use crate::opts::grammar::Grammar;
use crate::opts::parsed_args::{ParsedArgs, ParsedOpt};
use serde::Deserialize;
use std::mem::swap;
use eyre::{bail, Result};

#[derive(Clone, Debug, Deserialize)]
pub struct Gnu {
    /// Determines whether positional args must all appear last (after the last option).
    /// If this flag is set to false (the default) positional args can appear in any place, including before an option.
    ///
    /// (Parsing-only)
    #[serde(rename = "strict_positional_args", default = "bool::default")]
    pub strict_positional_args: bool,

    /// Determines whether positional arguments should be disambiguated from options by adding a `--` before them.
    #[serde(default = "bool::default")]
    pub disambiguate_positional_args: bool,

}

impl Grammar for Gnu {
    fn parse<I: IntoIterator<Item=String>>(&self, args: I) -> Result<ParsedArgs> {
        let mut state = GnuParserState::new(self);

        let mut arg_iter = args.into_iter();

        loop {
            let Some(arg) = arg_iter.next() else {
                break;
            };
            state.parse_next(arg)?;
        }

        Ok(ParsedArgs {
            options: state.options,
            positional: state.positional,
        })
    }

    fn generate<F: FnMut(String)>(&self, args: ParsedArgs, mut f: F) -> Result<()> {
        for arg in args.options {
            let ParsedOpt { name, values, .. } = arg;

            let prefix = if arg.short { "-" } else { "--" };
            match values.first() {
                Some(single_value) => {
                    if values.len() > 1 {
                        bail!("Multiple values are not supported yet")
                    }
                    f(format!("{prefix}{name}={single_value}"))
                }
                None => {
                    f(format!("{prefix}{name}"))
                }
            }
        }

        if self.disambiguate_positional_args {
            f("--".to_string());
        }

        for arg in args.positional {
            f(arg);
        };

        Ok(())
    }
}

pub fn split_kv(kv_arg: &str) -> (String, Option<String>) {
    let sign_pos = kv_arg.chars().position(|c| c == '=');
    let (name, value) = match sign_pos {
        None => (kv_arg.to_string(), None),
        Some(pos) => {
            let (name, eq_value) = kv_arg.split_at(pos);
            let value = &eq_value[1..];
            (name.to_string(), Some(value.to_string()))
        }
    };
    (name, value)
}

struct GnuParserState<'a> {
    grammar: &'a Gnu,
    options: Vec<ParsedOpt>,
    positional: Vec<String>,
    positional_only: bool,
}

impl<'a> GnuParserState<'a> {
    fn new(grammar: &'a Gnu) -> Self {
        Self {
            grammar,
            options: Vec::new(),
            positional: Vec::new(),
            positional_only: false,
        }
    }

    fn parse_next(&mut self, arg: String) -> Result<()> {
        if self.positional_only {
            return self.add_positional(arg);
        }

        // All arguments following the first '--' are positional arguments
        if !self.positional_only && arg == "--" {
            self.positional_only = true;
            return Ok(());
        }

        if arg.starts_with("--") {
            return self.add_long(&arg);
        }

        if arg.starts_with('-') {
            return self.add_short(arg);
        }

        self.add_unmarked(arg)
    }

    fn check_if_options_are_allowed(&self) -> Result<()> {
        if self.positional_only && self.grammar.strict_positional_args {
            bail!(
                "Option arguments cannot be mixed with positional arguments \
                 (positional arguments must strictly follow options)"
            );
        }
        Ok(())
    }

    fn add_unmarked(&mut self, arg: String) -> Result<()> {
        if self.grammar.disambiguate_positional_args {
            bail!("Positional arguments must be unambiguous add a '--' before the positional arguments")
        }

        self.add_positional(arg)
    }

    fn add_positional(&mut self, arg: String) -> Result<()> {
        self.positional.push(arg);
        Ok(())
    }

    fn add_short(&mut self, arg: String) -> Result<()> {
        let name = &arg[1..];
        let opt = ParsedOpt {
            name: name.to_string(),
            values: Vec::new(),
            short: true,
        };
        self.options.push(opt);
        Ok(())
    }

    fn add_long(&mut self, arg: &String) -> Result<()> {
        let (name, value) = split_kv(&arg[2..]);
        let opt = ParsedOpt {
            name,
            values: value.into_iter().collect(),
            short: false,
        };
        self.options.push(opt);
        Ok(())
    }
}
