use std::borrow::Cow;
use std::io::BufRead;
use eyre::{bail, Context, ContextCompat, eyre, Result};
use itertools::Itertools;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use tap::Pipe;

use crate::commands::common::Command;
use crate::commands::common::CommandParser;
use crate::commands::parser_exts::{InnerUnwrap, PairExt, PairsExt, SupportsQuoting};
use crate::opts::parsed_args::OptName;

#[derive(Parser)]
#[grammar = "grammar/compact.pest"]
struct InternalParser;

#[derive(Default)]
pub struct CompactCommandParser;

impl CommandParser for CompactCommandParser {
    fn parse_from_script_src<R: BufRead>(&self, script: R) -> Result<Vec<Command>> {
        let lines: Vec<String> = script.lines()
                .map_ok(|x| x.trim_start().to_string())
                .filter_ok(|x| !x.is_empty() && !x.starts_with('#'))
                .try_collect()?;
        self.parse_from_args(lines.iter())
    }

    fn parse_from_script(&self, script: &str) -> Result<Vec<Command>> {
        // TODO: Support quoting inside raw_value_arg, so that we can have newlines embedded in scripts
        let lines = script.lines()
                .map(|x| x.trim_start().to_string())
                .filter(|x| !x.is_empty() && !x.starts_with('#'));
        self.parse_from_args(lines)
    }

    fn parse_from_args<'a, I, S>(&self, commands: I) -> Result<Vec<Command>>
        where S: AsRef<str>,
              I: Iterator<Item=S>
    {
        commands
                .map(|cmd| {
                    let cmd = cmd.as_ref();
                    parse_command(cmd)
                            .wrap_err_with(|| format!("Failed to parse command: {cmd}"))
                })
                .collect()
    }
}

fn parse_command(command: &str) -> Result<Command> {
    let command_pair = InternalParser::parse(Rule::command_input, command)
            .wrap_err("Bad syntax for compact command")?
            .next()
            .map_single_wrapped() // command
            .map_single_wrapped() // typed command
            .wrap_err("Missing typed command")?;

    let mut matches = command_pair.clone().into_inner();
    let option: OptName = matches.expect_rule(Rule::option_name)?
            .parse_quoted()?
            .try_into()?;

    let value_rules = [Rule::set_values, Rule::add_values, Rule::repeat_values];
    let values = matches
            .attempt_rules(&value_rules)
            .map(parse_value_clause)
            .transpose()?
            .unwrap_or_default();

    // Convert vector items from Cow<str> to String (using move when possible)
    let args = values.args.into_iter().map(Cow::into_owned).collect_vec();

    let command = match (&command_pair.as_rule(), values.modifier) {
        (Rule::set_command, Modifier::Append { delimiter }) =>
            Command::Append {
                option,
                delimiter: delimiter.unwrap_or("".into()).into_owned(),
                items: args,
            },
        (Rule::set_command, _) => Command::Set { option, values: args },
        (Rule::add_command, _) => Command::Add { option, values: args },
        (Rule::remove_command, _) => Command::Remove { option },
        (Rule::repeat_command, _) => Command::RepeatedAdd { option, values: args },
        (_, _) => bail!("Unknown command rule detected: {:?}", &command_pair.as_rule()),
    };

    Ok(command)
}

fn parse_value_clause(values_clause: Pair<Rule>) -> Result<ValueClause> {
    let mut matches = values_clause.clone().into_inner();

    // Parse the modifier first (coming before the '=' sign)
    let modifier: Modifier = matches
            .attempt_rules(&[Rule::append_mod, Rule::multi_arg_mod])
            .pipe(parse_modifier)?;

    // Parse value args (coming after the '=' sign)
    let Some(value_args_pair) = matches.next() else {
        bail!("arguments are expected but missing")
    };
    let args = match value_args_pair.as_rule() {
        Rule::raw_value_arg => {
            // Collect the single argument into a vector
            vec![value_args_pair.as_str().into()]
        }
        Rule::value_args => {
            // Collect space-separated, quotable, arguments into a vector
            value_args_pair
                    .into_inner().filter(|pair| pair.as_rule() == Rule::value_arg)
                    .map(|pair| pair.parse_quoted()).try_collect()?
        }
        _ => bail!("Unexpected rule for arguments: {:?}", value_args_pair.as_rule()),
    };

    Ok(ValueClause { modifier, args })
}

fn parse_modifier(modifier: Option<Pair<Rule>>) -> Result<Modifier> {
    let Some(modifier) = modifier else {
        return Ok(Modifier::None);
    };

    let parsed = match modifier.as_rule() {
        Rule::append_mod => {
            let delimiter = modifier.into_inner()
                    .attempt_rule(Rule::append_delimiter)
                    .and_then(|pair| pair.map_single_wrapped())
                    .map(|pair| pair.parse_quoted())
                    .transpose()
                    .wrap_err_with(|| eyre!("Bad append delimiter"))?;

            Modifier::Append { delimiter }
        }
        Rule::multi_arg_mod => Modifier::MultiArg,
        _ => bail!("Unexpected modifier '{}'", modifier.as_str()),
    };

    Ok(parsed)
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum Modifier<'i> {
    #[default]
    None,
    MultiArg,
    Append { delimiter: Option<Cow<'i, str>> },
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
struct ValueClause<'i> {
    modifier: Modifier<'i>,
    args: Vec<Cow<'i, str>>,
}

impl SupportsQuoting for Rule {
    #[inline]
    fn quoted_rule() -> Self {
        Rule::quoted
    }
}

