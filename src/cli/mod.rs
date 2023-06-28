mod dialect;
mod deserialize_from_args;

use crate::opts::grammar::Grammar;
use clap::{Parser, Subcommand, ValueEnum};
use clap::builder::{PossibleValue, ValueParserFactory};
use clio::Input;
use enum_dispatch::enum_dispatch;

pub use crate::cli::dialect::Dialect;

// Derive arguments with clap
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Edit {
        /// Input file, use '-' for stdin
        #[arg(short('f'), long("file"), value_parser, default_value = "-")]
        input: Input,

        #[arg(short('I'), long, default_value = "gnu")]
        input_dialect: Dialect,

        #[arg(short('O'), long, default_value = "gnu")]
        output_dialect: Dialect,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Foo {
    Gnu,
    Posix,
}
