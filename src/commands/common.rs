use std::io::{BufRead};
use eyre::Result;
use crate::opts::parsed_args::{OptName, ParsedArgs, ParsedOpt};
use crate::util::vec::PushExt;

pub trait CommandParser {
    fn parse_from_script_src<R: BufRead>(&self, script: R) -> Result<Vec<Command>>;
    fn parse_from_script(&self, script: &str) -> Result<Vec<Command>>;
    fn parse_from_args<'a, I, S>(&self, commands: I) -> Result<Vec<Command>>
        where S: AsRef<str>,
              I: Iterator<Item=S>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Set { option: OptName, values: Vec<String> },
    Add { option: OptName, values: Vec<String> },
    Remove { option: OptName },
    Append { option: OptName, delimiter: String, items: Vec<String> },
    RepeatedAdd { option: OptName, values: Vec<String> },
}

impl Command {
    pub fn apply(self, args: &mut ParsedArgs) {
        match self {
            Command::Set { option, values } => {
                args.set_last_option(option, values);
            }
            Command::Add { option, values } => {
                args.options.push(ParsedOpt { name: option, values })
            }
            Command::Remove { option } => {
                args.remove_all_options(option)
            }
            Command::RepeatedAdd { option, values: value_for_each } => {
                for value in value_for_each {
                    args.options.push(ParsedOpt { name: option.clone(), values: vec![value] })
                }
            }
            Command::Append { option, delimiter, items  } => {
                args.update_last_option(option, |values| {
                    let first_value = values.ensure_first_or_default();

                    if delimiter.is_empty() {
                        first_value.extend(items);
                    } else {
                        for item in items {
                            first_value.push_str(delimiter.as_str());
                            first_value.push_str(&item);
                        }
                    }
                });
            }
        }
    }
}

