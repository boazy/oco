use std::io::BufRead;
use eyre::{bail, Context, ContextCompat, eyre, Result};
use itertools::Itertools;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;

use crate::commands::{Command, CommandParser};
use crate::commands::parser_exts::{InnerUnwrap, PairExt, PairsExt, SupportsQuoting};
use crate::opts::parsed_args::OptName;

#[derive(Parser)]
#[grammar = "grammar/full.pest"]
struct InternalParser;

#[derive(Default)]
pub struct FullCommandParser;

impl CommandParser for FullCommandParser {
    fn parse_from_script_src<R: BufRead>(&self, mut script: R) -> Result<Vec<Command>> {
        let mut script_str = String::new();
        script.read_to_string(&mut script_str)
                .wrap_err("Failed to read script file")?;
        self.parse_from_script(&script_str)
    }

    fn parse_from_script(&self, script: &str) -> Result<Vec<Command>> {
        InternalParser::parse(Rule::commands, script)
                .wrap_err("Cannot parse full command script")?
                .next().wrap_err("No commands found in input")?
                .into_inner()
                .map(parse_command)
                .collect_commands()
    }

    fn parse_from_args<'a, I, S>(&self, commands: I) -> Result<Vec<Command>> where S: AsRef<str>, I: Iterator<Item=S> {
        commands
                .map(|command| {
                    let command = InternalParser::parse(
                        Rule::command, command.as_ref())
                            .wrap_err_with(|| eyre!("Cannot parse command: {}", command.as_ref()))
                            .and_then(|mut matches|
                                    matches.next().ok_or_else(|| eyre!("No command found in input: {}", command.as_ref()))
                            )?;

                    parse_command(command)
                })
                .collect_commands()
    }
}

fn parse_command(command: Pair<Rule>) -> Result<Option<Command>> {
    let rule = command.as_rule();
    let mut matches = command.into_inner();

    let cmd = match rule {
        Rule::set_command => {
            Command::Set {
                option: matches.expect_option_name()?,
                values: matches.read_values()?,
            }
        }
        Rule::add_command => {
            Command::Add {
                option: matches.expect_option_name()?,
                values: matches.read_values()?,
            }
        }
        Rule::radd_command => {
            Command::RepeatedAdd {
                option: matches.expect_option_name()?,
                values: matches.read_values()?,
            }
        }
        Rule::remove_command => {
            Command::Remove { option: matches.expect_option_name()? }
        }
        Rule::append_command => {
            Command::Append {
                delimiter: matches
                        .attempt_rule(Rule::quoted)
                        .map(|pair| pair.parse_quoted_into_string())
                        .unwrap_or(Ok("".to_string()))?, // Default to empty string if no delimiter is specified
                option: matches.expect_option_name()?,
                items: matches.read_values()?,
            }
        }
        Rule::EOI => return Ok(None), // Ignore EOI
        _ => bail!("Unknown command rule: {rule:?}")
    };

    Ok(Some(cmd))
}

trait CommandsIterator
{
    fn collect_commands(&mut self) -> Result<Vec<Command>>;
}

impl<I: Iterator<Item=Result<Option<Command>>>> CommandsIterator for I {
    fn collect_commands(&mut self) -> Result<Vec<Command>> {
        self
                .filter_map(|result| result.transpose())
                .try_collect().wrap_err("Failed to parse full command script")
    }
}

trait ParserPairsExt {
    fn expect_option_name(&mut self) -> Result<OptName>;
    fn read_values(&mut self) -> Result<Vec<String>>;
}

impl<'i> ParserPairsExt for Pairs<'i, Rule> {
    fn expect_option_name(&mut self) -> Result<OptName> {
        self
                .expect_rule(Rule::name)?
                .parse_quoted()?
                .try_into()
    }

    fn read_values(&mut self) -> Result<Vec<String>> {
        self
                .filter(|pair| pair.as_rule() == Rule::value_arg)
                .map(|pair| pair
                        .map_single_wrapped()
                        .wrap_err("Expected quotable value")?
                        .parse_quoted_into_string())
                .try_collect()
    }
}

impl SupportsQuoting for Rule {
    #[inline]
    fn quoted_rule() -> Self {
        Rule::quoted
    }
}

#[cfg(test)]
mod test {
    use paste::paste;
    use crate::commands::{Command, CommandParser, FullCommandParser};
    use crate::commands::Command::{Append, RepeatedAdd, Set};
    use crate::util::testing::opts::sv;
    use crate::util::testing::opts::name::{short, long};

    static PARSER: FullCommandParser = FullCommandParser {};

    fn test_parse_args(input_cmds: &[&str], expected_result: &[Command]) -> eyre::Result<()> {
        let result = PARSER.parse_from_args(input_cmds.iter())?;
        assert_eq!(result, expected_result);
        Ok(())
    }

    fn test_parse_script(script: &str, expected_result: &[Command]) -> eyre::Result<()> {
        let result = PARSER.parse_from_script(script)?;
        assert_eq!(result, expected_result);
        Ok(())
    }

    fn test_parse_script_src(script_src: &str, expected_result: &[Command]) -> eyre::Result<()> {
        let result = PARSER.parse_from_script_src(script_src.as_bytes())?;
        assert_eq!(result, expected_result);
        Ok(())
    }

    macro_rules! test_cmds {
        ($name:ident, $($cmd_str:literal),+ => $($cmd:expr),+) => {
            paste! {
                #[test]
                fn [<parse_ $name _with_args>]() -> ::eyre::Result<()> {
                    let input_cmds = [$($cmd_str,)*];
                    let expected_result = [$($cmd,)*];
                    test_parse_args(&input_cmds, &expected_result)
                }

                #[test]
                fn [<parse_ $name _with_script>]() -> ::eyre::Result<()> {
                    let input_script = [$($cmd_str,)*].join("\n");
                    let expected_result = [$($cmd,)*];
                    test_parse_script(input_script.as_str(), &expected_result)
                }

                #[test]
                fn [<parse_ $name _with_script_src>]() -> ::eyre::Result<()> {
                    let input_script = [$($cmd_str,)*].join("\n");
                    let expected_result = [$($cmd,)*];
                    test_parse_script_src(input_script.as_str(), &expected_result)
                }
            }
        }
    }

    macro_rules! test_script {
        ($name:ident, $script:literal => $($cmd:expr),+) => {
            paste! {
                #[test]
                fn [<parse_ $name>]() -> ::eyre::Result<()> {
                    let expected_result = [$($cmd,)*];
                    test_parse_script($script, &expected_result)
                }

                #[test]
                fn [<parse_ $name _from_src>]() -> ::eyre::Result<()> {
                    let expected_result = [$($cmd,)*];
                    test_parse_script_src($script, &expected_result)
                }
            }
        }
    }

    #[test]
    fn empty_parse_from_empty_args() {
        let result = PARSER.parse_from_args(Vec::<String>::new().iter());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn empty_parse_from_empty_script() {
        let result = PARSER.parse_from_script("");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn empty_parse_from_empty_script_src() {
        let result = PARSER.parse_from_script_src("".as_bytes());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    test_script!(script_with_comments,
        r###"
        #!/bin/bash

        set test=123

        # Comment
          # Indented Comment
            append '; world ; ' foo=1 # Indented command
        append ',' foo=2
        radd a="quoted 'here' too" # Not everything

        "### =>
        Set { option: long!("test"), values: sv!["123"] },
        Append { option: long!("foo"), items: sv!["1"], delimiter: "; world ; ".to_string() },
        Append { option: long!("foo"), items: sv!["2"], delimiter: ",".to_string(), },
        RepeatedAdd { option: short!('a'), values: sv!["quoted 'here' too"] }
    );

    test_script!(mixed_whitespace, " \t set a \t= \t bar\t\t# Mixed whitespace" =>
        Set {option: short!('a'), values: sv!["bar"]}
    );

    test_cmds!(multiple_sets, "set ignore=foo", "set ignore", "set a=1 2 three", "set b", "set long=abc def" =>
        Set { option: long!("ignore"), values: sv!["foo"] },
        Set { option: long!("ignore"), values: sv![] },
        Set { option: short!('a'), values: sv!["1", "2", "three"] },
        Set { option: short!('b'), values: sv![] },
        Set { option: long!("long"), values: sv!["abc", "def"] }
    );
}