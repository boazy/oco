mod dialect;
mod deserialize_from_args;

use clap::{Args, Parser};
use clio::{Input, Output};


pub use crate::cli::dialect::Dialect;

// Derive arguments with clap
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Input file, use '-' for stdin
    #[arg(short('i'), long, value_parser, default_value = "-")]
    pub input: Input,

    /// Output file, use '-' for stdout
    #[arg(short('o'), long, value_parser, default_value = "-")]
    pub output: Output,

    #[arg(short('I'), long, default_value = "gnu")]
    pub input_dialect: Dialect,

    #[arg(short('O'), long, default_value = "gnu")]
    pub output_dialect: Dialect,

    #[clap(flatten)]
    pub commands: CommandSpec,
}

#[derive(Args, Debug)]
pub struct CommandSpec {
    /// Use full script language syntax for argument processing
    ///
    /// If this flag is not specified, the compact syntax will be used.
    #[arg(short('l'), default_value = "false")]
    pub full_script_syntax: bool,

    #[arg(short('f'), long("file"), value_parser, group = "commands")]
    pub command_file: Option<Input>,

    #[arg(short('c'), long, default_value = "", group = "commands")]
    pub with_commands: String,

    #[arg(group="commands", value_name = "COMMAND")]
    pub args: Vec<String>,
}