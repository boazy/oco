mod cli;
mod opts;
mod read_args;
mod util;
mod commands;

use clap::Parser;
use cli::Cli;
use eyre::{Context, Result};
use opts::grammar::Grammar;
use std::io::{BufReader, BufWriter, Write};

use crate::cli::{CommandSpec, Dialect};
use crate::read_args::read_args;
use crate::commands::{Command, CommandParser, CompactCommandParser, FullCommandParser};
use crate::util::shell;


fn main() {
    color_eyre::install().expect("Failed to install color_eyre");

    let cli = Cli::parse();
    let args = read_args(BufReader::new(cli.input));
    let mut parsed = <Dialect as Grammar>::parse(&cli.input_dialect, args)
            .expect("Failed to parse input arguments");

    let commands = parse_commands(cli.commands)
            .expect("Failed to parse commands");

    for command in commands {
        command.apply(&mut parsed)
    }

    let mut output = BufWriter::new(cli.output);

    let mut first = true;
    <Dialect as Grammar>::generate(&cli.output_dialect, parsed, |arg| {
        let quoted_arg = shell::quote(arg.as_str());
        if first {
            first = false;
            write!(output, "{quoted_arg}").expect("Failed to write output argument");
        } else {
            write!(output, " {quoted_arg}").expect("Failed to write output argument");
        }
    }).expect("Failed to generate output arguments");
}

fn parse_commands(commands: CommandSpec) -> Result<Vec<Command>> {
    if commands.full_script_syntax {
        FullCommandParser::default().parse_commands(commands)
    } else {
        CompactCommandParser::default().parse_commands(commands)
    }
}

trait CommandParserExt {
    fn parse_commands(&self, commands: CommandSpec) -> Result<Vec<Command>>;
}

impl<P: CommandParser> CommandParserExt for P {
    fn parse_commands(&self, commands: CommandSpec) -> Result<Vec<Command>> {
        if let Some(command_file) = commands.command_file {
            let path = command_file.path().to_string_lossy().into_owned();
            self
                    .parse_from_script_src(BufReader::new(command_file))
                    .wrap_err_with(|| format!("Failed to parse commands from script file: {path}"))
        } else if !commands.with_commands.is_empty() {
            self
                    .parse_from_script(commands.with_commands.as_str())
                    .wrap_err("Failed to parse commands from script")
        } else {
            self
                    .parse_from_args(commands.args.iter())
                    .wrap_err("Failed to parse commands from arguments")
        }
    }
}
