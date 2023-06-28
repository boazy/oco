use std::backtrace;
use std::collections::BTreeMap;
use clap::builder::{ValueParser, ValueParserFactory};
use color_eyre::Help;
use enum_dispatch::enum_dispatch;
use enum_dispatch_clone::EnumDispatchClone;
use crate::cli::deserialize_from_args::deserialize_from_args;
use crate::opts::grammar::Gnu;
use eyre::{bail, Context, eyre, Result};

#[derive(Debug, PartialEq, Eq, EnumDispatchClone)]
#[enum_dispatch(Grammar)]
pub enum Dialect {
    Gnu,
}

impl ValueParserFactory for Dialect {
    type Parser = ValueParser;

    fn value_parser() -> Self::Parser {
        parse_dialect.into()
    }
}

fn parse_dialect(value: &str) -> Result<Dialect> {
    // Parse dialect and arguments
    // Should be in the format {dialect}:{arg1},{arg2},...
    let (name, args) = parse_dialect_string(value);

    if name == "gnu" {
        let grammar: Gnu = deserialize_from_args(args.into_iter())
                .map_err(|e| {
                    let args_str = &value.get(name.len() + 1..).unwrap_or("");
                    eyre::Error::msg(
                        format!("Cannot parse grammar arguments for {name} ('{args_str}'): {e}")
                    ).with_error(|| e)
                })?;
        Ok(grammar.into())
    } else {
        bail!("Unknown dialect {name}");
    }
}

fn parse_dialect_string(value: &str) -> (&str, Vec<(&str, Option<&str>)>) {
    let (name, args) = value.split_once(":").unwrap_or((value, ""));

    if args.is_empty() {
        return (name, Vec::new());
    }

    // Return arguments as map
    let arg_map = args.split(",").map(|arg| {
        match arg.split_once("=") {
            None => (arg, None),
            Some((key, value)) => (key, Some(value))
        }
    });

    (name, arg_map.collect())
}
