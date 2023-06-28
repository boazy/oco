mod cli;
mod opts;
mod read_args;

use std::fs::remove_dir_all;
use crate::cli::Command;
use crate::read_args::read_args;
use clap::Parser;
use cli::Cli;
use opts::grammar::Grammar;
use std::io::{BufReader, BufWriter, Write};
use std::iter;
use clap::builder::TypedValueParser;
use cli::Dialect;

fn main() {
    color_eyre::install().expect("Failed to install color_eyre");

    let mut cli = Cli::parse();
    match cli.command {
        Command::Edit { input, input_dialect, output_dialect } => {
            let args = read_args(BufReader::new(input));
            let parsed = input_dialect.parse(args)
                    .expect("Failed to parse input arguments");
            println!("{:?}", parsed);

            let mut output = BufWriter::new(std::io::stdout());

            let mut first = true;
            output_dialect.generate(parsed, |arg| {
                let quoted_arg = shlex::quote(arg.as_str());
                if first {
                    first = false;
                    write!(output, "{quoted_arg}").expect("Failed to write output argument");
                } else {
                    write!(output, " {quoted_arg}").expect("Failed to write output argument");
                }
            }).expect("Failed to generate output arguments");
        }
    }
}
