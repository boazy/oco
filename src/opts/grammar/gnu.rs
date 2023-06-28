use crate::opts::grammar::Grammar;
use crate::opts::parsed_args::{ParsedArgs, ParsedOpt};
use derivative::Derivative;
use derive_new::new;
use serde::Deserialize;

use eyre::{bail, Result};
use crate::opts::parsed_args::OptName::{Long, Short};

#[derive(Clone, Debug, Derivative, Deserialize, Eq, PartialEq)]
#[derivative(Default)]
pub struct Gnu {
    #[serde(default = "Default::default")]
    pub positional: PositionalArgumentsMode,

    /// Determines whether positional arguments will be disambiguated from options by adding a `--` before them,
    ///
    /// (Generating-only)
    #[serde(default = "always_true")]
    #[derivative(Default(value = "true"))]
    pub explicit_positional: bool,

    /// Determines whether short options can be grouped together.
    #[serde(default = "always_true")]
    #[derivative(Default(value = "true"))]
    pub grouping: bool,

    /// Determines how the long option argument is specified
    #[serde(default = "Default::default")]
    pub long_arg: LongOptionArgumentFormat,
}


#[derive(Copy, Clone, Default, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LongOptionArgumentFormat {
    #[default]
    /// The argument is specified as `--option=argument`
    Equals,

    /// The argument is specified as `--option argument`
    ///
    /// This could result in parsing ambiguity when [disambiguate_positional_args](Gnu::disambiguate_positional_args) is
    /// set to false (the default value). To allow parsing in this case, you must provide a vocabulary.
    Space,
}

#[derive(Copy, Clone, Default, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PositionalArgumentsMode {
    Forbidden,

    /// Positional arguments are allowed anywhere, including before the first option.
    ///
    /// This means that option arguments separated by space, cannot be distinguished from positional arguments, and will
    /// be treated as positional arguments, unless a vocabulary is provided.
    Free,

    /// Positional arguments are allowed only after an explicit `--` that clear disambiguates them from option
    /// arguments. Every unmarked argument (an argument not not prefixed with a `-` or `--`) that appears before the
    /// disambiguating `--` is treated as an argument to the previous option. Unmarked arguments before first option are
    /// not allowed with this option.
    #[default]
    Explicit,
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
        for opt in args.options {
            let ParsedOpt { name, values } = opt;
            match values.split_first() {
                // No arguments
                None => match name {
                    Short(name) =>
                        f(format!("-{name}")),
                    Long(name) =>
                        f(format!("--{name}")),
                }
                // Single argument value
                Some((single_value, &[])) => match name {
                    Short(name) => {
                        f(format!("-{name}"));
                        f(single_value.to_string())
                    }
                    Long(name) =>
                        self.generate_first_long_arg(name, single_value, &mut f),
                }
                Some((first_value, rest_of_values)) => {
                    match name {
                        Short(name) => {
                            f(format!("-{name}"));
                            for value in values {
                                f(value);
                            }
                        }
                        Long(name) => {
                            self.generate_first_long_arg(name, first_value, &mut f);
                            for value in rest_of_values {
                                f(value.to_string());
                            }
                        }
                    }
                }
            }
        }

        // No positional arguments
        if args.positional.is_empty() {
            return Ok(());
        }

        if self.explicit_positional {
            f("--".to_string());
        }

        for arg in args.positional {
            f(arg);
        };

        Ok(())
    }
}

impl Gnu {
    fn generate_first_long_arg<F: FnMut(String)>(&self, name: String, value: &str, f: &mut F) {
        match self.long_arg {
            LongOptionArgumentFormat::Equals =>
                f(format!("--{name}={value}")),
            LongOptionArgumentFormat::Space => {
                f(format!("--{name}"));
                f(value.to_string())
            }
        }
    }
}

fn split_kv(kv_arg: &str) -> (String, Option<String>) {
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

#[derive(new)]
struct GnuParserState<'a> {
    grammar: &'a Gnu,
    #[new(default)]
    options: Vec<ParsedOpt>,
    #[new(default)]
    positional: Vec<String>,
    #[new(default)]
    always_treat_as_positional: bool,
}


impl<'a> GnuParserState<'a> {
    fn parse_next(&mut self, arg: String) -> Result<()> {
        if self.always_treat_as_positional {
            return self.add_positional(arg);
        }

        // All arguments following the first '--' are positional arguments
        if arg == "--" {
            if self.grammar.positional == PositionalArgumentsMode::Forbidden {
                bail!("Positional arguments are not allowed");
            }
            self.always_treat_as_positional = true;
            return Ok(());
        }

        if arg.starts_with("--") {
            return self.add_long(&arg);
        }

        if arg.starts_with('-') {
            return self.add_short(&arg);
        }

        self.add_unmarked(arg)
    }

    fn add_unmarked(&mut self, arg: String) -> Result<()> {
        use crate::opts::grammar::gnu::PositionalArgumentsMode::*;

        match (self.grammar.positional, self.options.last_mut()) {
            (Free, _) =>
            // In free mode, we always treat unmarked arguments as positional.
            // TODO: Adding vocabulary would change that!
                self.add_positional(arg),
            (_, Some(last_option)) => {
                // In non-free mode, treat the argument as a value for the last option before it (if there is any).
                last_option.values.push(arg);
               Ok(())
            }
            (Forbidden, None) =>
                bail!("Positional arguments are not allowed"),
            (Explicit, None) =>
                bail!("Positional arguments must be unambiguous. \
                           Add a '--' before specifying positional arguments"),
        }
    }

    fn add_positional(&mut self, arg: String) -> Result<()> {
        self.positional.push(arg);
        Ok(())
    }

    fn add_short(&mut self, arg: &str) -> Result<()> {
        let option = &arg[1..];

        // A stand-alone - character is a positional argument, not an option
        if option.is_empty() {
            self.add_unmarked(arg.to_string())?;
        }

        let mut chars = option.chars();

        if !self.grammar.grouping {
            if option.len() > 1 {
                bail!("Short options cannot be grouped together");
            }
            self.add_short_char(chars.next().unwrap());
            return Ok(());
        }

        // Add every character in option string, so '-abc' will be treated as '-a -b -c'
        for c in chars {
            self.add_short_char(c);
        }
        Ok(())
    }

    fn add_short_char(&mut self, c: char) {
        self.options.push(ParsedOpt {
            name: Short(c),
            values: Vec::new(),
        });
    }

    fn add_long(&mut self, arg: &str) -> Result<()> {
        let (name, value) = split_kv(&arg[2..]);
        self.options.push(ParsedOpt {
            name: Long(name),
            values: value.into_iter().collect(),
        });
        Ok(())
    }
}


fn always_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;
    use eyre::Result;
    use crate::util::testing::{assert_err, assert_err_contains};
    use crate::util::testing::opts::sv;
    use crate::util::testing::opts::parsed::{short, long};

    use crate::opts::grammar::{Gnu, Grammar};
    use crate::opts::parsed_args::ParsedArgs;
    use crate::opts::grammar::gnu::{LongOptionArgumentFormat, PositionalArgumentsMode};

    #[test]
    fn test_split_kv() {
        assert_eq!(super::split_kv("foo"), ("foo".to_string(), None));
        assert_eq!(super::split_kv("foo="), ("foo".to_string(), Some("".to_string())));
        assert_eq!(super::split_kv("foo=bar"), ("foo".to_string(), Some("bar".to_string())));

        // Corner cases
        assert_eq!(super::split_kv(""), ("".to_string(), None));
        assert_eq!(super::split_kv("="), ("".to_string(), Some("".to_string())));
        assert_eq!(super::split_kv("1=2"), ("1".to_string(), Some("2".to_string())));
    }

    #[test]
    fn test_parse_normal() -> Result<()> {
        let grammar = Gnu::default();

        // Without positional arguments
        let parsed = grammar.parse_arr(&[
            "-a", "a1", "a2", "-bc", "c1", "c2", "c3", "--foo=bar", "baz", "bug"
        ])?;
        assert_eq!(parsed.options, vec![
            short!('a', "a1", "a2"),
            short!('b'),
            short!('c', "c1", "c2", "c3"),
            long!("foo", "bar", "baz", "bug")],
        );
        assert!(parsed.positional.is_empty());

        // With positional arguments
        let parsed = grammar.parse_arr(&[
            "-a", "a1", "a2", "-bc", "c1", "c2", "c3", "--foo=bar", "baz", "bug", "--", "pos1", "pos2"
        ])?;
        assert_eq!(parsed.options, vec![
            short!('a', "a1", "a2"),
            short!('b'),
            short!('c', "c1", "c2", "c3"),
            long!("foo", "bar", "baz", "bug")],
        );
        assert_eq!(parsed.positional, vec!["pos1", "pos2"]);

        // One short option
        let parsed = grammar.parse_arr(&["-z"])?;
        assert_eq!(parsed.options, vec![short!('z')]);
        assert!(parsed.positional.is_empty());

        // One long option
        let parsed = grammar.parse_arr(&["--z"])?;
        assert_eq!(parsed.options, vec![long!("z")]);
        assert!(parsed.positional.is_empty());

        // All args after -- are treated as positional
        let parsed = grammar.parse_arr(&["-a", "arg1", "--", "-f", "--foo"])?;
        assert_eq!(parsed.options, vec![short!('a', "arg1")]);
        assert_eq!(parsed.positional, vec!["-f", "--foo"]);

        // Only positional arguments, starting with --
        let parsed = grammar.parse_arr(&["--", "pos1", "pos2"])?;
        assert!(parsed.options.is_empty());
        assert_eq!(parsed.positional, vec!["pos1", "pos2"]);

        Ok(())
    }

    #[test]
    fn test_parse_free() -> Result<()> {
        let grammar = Gnu { positional: PositionalArgumentsMode::Free, ..Gnu::default() };

        // Without arguments separator
        let parsed = grammar.parse_arr(&["pos1", "-a", "pos2", "-bc", "--foo=bar", "pos3", "pos4"])?;
        assert_eq!(parsed.options, vec![short!('a'), short!('b'), short!('c'), long!("foo", "bar")]);
        assert_eq!(parsed.positional, vec!["pos1", "pos2", "pos3", "pos4"]);

        // With argument separator (all args after -- are treated as positional)
        let parsed = grammar.parse_arr(&["pos1", "-a", "-", "--", "-bc", "--foo=bar", "pos2"])?;
        assert_eq!(parsed.options, vec![short!('a')]);
        assert_eq!(parsed.positional, vec!["pos1", "-", "-bc", "--foo=bar", "pos2"]);

        // Only positional arguments
        let parsed = grammar.parse_arr(&["pos1", "pos2"])?;
        assert!(parsed.options.is_empty());
        assert_eq!(parsed.positional, vec!["pos1", "pos2"]);

        // Only positional arguments, starting with --
        let parsed = grammar.parse_arr(&["--", "--", "pos1", "pos2"])?;
        assert!(parsed.options.is_empty());
        assert_eq!(parsed.positional, vec!["--", "pos1", "pos2"]);

        Ok(())
    }

    #[test]
    fn test_parse_empty_args() -> Result<()> {
        let grammar = Gnu::default();
        let parsed = grammar.parse_arr(&[])?;
        assert!(parsed.options.is_empty());
        assert!(parsed.positional.is_empty());
        Ok(())
    }

    #[test]
    fn test_forbidden_pos_arg_fails() {
        let grammar = Gnu::default(); // Positional arguments are not forbidden, but must be explicit
        assert_err_contains!(grammar.parse_arr(&["pos1", "-a"]), "Positional arguments must be unambiguous");

        let forbidden_grammar = Gnu { positional: PositionalArgumentsMode::Forbidden, ..Default::default() };
        assert_err_contains!(forbidden_grammar.parse_arr(&["pos1", "-a"]), "Positional arguments are not allowed");

        let forbidden_grammar = Gnu { positional: PositionalArgumentsMode::Forbidden, ..Default::default() };
        assert_err_contains!(forbidden_grammar.parse_arr(&["-a", "--"]), "Positional arguments are not allowed");
        assert_err_contains!(forbidden_grammar.parse_arr(&["-a", "--", "-x"]), "Positional arguments are not allowed");
        assert_err_contains!(forbidden_grammar.parse_arr(&["--"]), "Positional arguments are not allowed");
        assert_err_contains!(forbidden_grammar.parse_arr(&["--", "-x"]), "Positional arguments are not allowed");
    }

    #[test]
    fn test_parse_edge_cases() -> Result<()> {
        let grammar = Gnu::default();

        // Empty string ("") is treated as positional and fails
        assert_err!(grammar.parse_arr(&["", "-a"]));

        // "-" has no option name and is therefore treated as positional (and fails)
        assert_err!(grammar.parse_arr(&["", "-a"]));

        // 1st argument   "-o"  treated as option
        // 2nd argument   ""    treated as 1st (main) option argument
        // 3rd argument   "-"   treated as 2nd option argument
        // 4th argument   "--"  treated as argument separator (and ignored)
        // 5th argument   "--"  separator already encountered, so it is treated as positional
        let parsed = grammar.parse_arr(&["-o", "", "-", "--", "--"])?;
        assert_eq!(parsed.options, vec![short!('o', "", "-")]);
        assert_eq!(parsed.positional, vec!["--"]);

        // Only an argument separator
        let parsed = grammar.parse_arr(&["--"])?;
        assert!(parsed.options.is_empty());
        assert!(parsed.positional.is_empty());

        // Option arguments and an argument separator without postional arguments
        let parsed = grammar.parse_arr(&["--foo", "-f", "bar", "--"])?;
        assert_eq!(parsed.options, vec![long!("foo"), short!('f', "bar")]);
        assert!(parsed.positional.is_empty());

        Ok(())
    }

    #[test]
    fn test_parse_free_edge_cases() -> Result<()> {
        let grammar = Gnu { positional: PositionalArgumentsMode::Free, ..Default::default() };

        // 1st argument   ""    an empty string (treated as positional)
        // 2nd argument   "-"   treated as positional (no option name)
        // 3rd argument   "--"  treated as argument separator (and ignored)
        // 4th argument   "--"  separator already encountered, so it is treated as positional
        let parsed = grammar.parse_arr(&["", "-", "--", "--"])?;
        assert!(parsed.options.is_empty());
        assert_eq!(parsed.positional, vec!["", "-", "--"]);

        // Only an argument separator
        let parsed = grammar.parse_arr(&["--"])?;
        assert!(parsed.options.is_empty());
        assert!(parsed.positional.is_empty());

        Ok(())
    }


    #[test]
    fn test_parse_no_grouping() -> Result<()> {
        let grammar = Gnu { grouping: false, ..Default::default() };

        // Ok: no grouping
        let parsed = grammar.parse_arr(&["-a", "arg1", "--foo", "-b"])?;
        assert_eq!(parsed.options, vec![short!('a', "arg1"), long!("foo"), short!('b')]);
        assert!(parsed.positional.is_empty());

        // Fail: no grouping
        assert_err_contains!(grammar.parse_arr(&["-ab"]), "cannot be grouped");

        Ok(())
    }

    #[test]
    fn test_generate_explicit() -> Result<()> {
        let equals_grammar = Gnu::default();

        // Positional args, long option with single argument, short option with no arguments
        let result = equals_grammar.generate_vec(ParsedArgs {
            options: vec![long!("long1", "42"), short!('s')],
            positional: sv!["pos1", "pos2"],
        })?;
        assert_eq!(result, vec!["--long1=42", "-s", "--", "pos1", "pos2"]);

        // No positional args, long option with no arguments, short option with single argument,
        // another short option with no arguments.
        let result = equals_grammar.generate_vec(ParsedArgs {
            options: vec![long!("foo"), short!('a', "42"), short!('b')],
            positional: vec![],
        })?;
        assert_eq!(result, vec!["--foo", "-a", "42", "-b"]);

        let result = equals_grammar.generate_vec(ParsedArgs {
            options: vec![short!('a', "a", "b", "c"), long!("foo", "1", "2")],
            positional: sv!["x"],
        })?;
        assert_eq!(result, vec!["-a", "a", "b", "c", "--foo=1", "2", "--", "x"]);

        Ok(())
    }

    #[test]
    fn test_generate_explicit_with_spaces() -> Result<()> {
        let equals_grammar = Gnu { long_arg: LongOptionArgumentFormat::Space, ..Default::default() };

        // Positional args, long option with single argument, short option with no arguments
        let result = equals_grammar.generate_vec(ParsedArgs {
            options: vec![long!("long1", "42"), short!('s')],
            positional: sv!["pos1", "pos2"],
        })?;
        assert_eq!(result, vec!["--long1", "42", "-s", "--", "pos1", "pos2"]);

        // No positional args, long option with no arguments, short option with single argument,
        // another short option with no arguments.
        let result = equals_grammar.generate_vec(ParsedArgs {
            options: vec![long!("foo"), short!('a', "42"), short!('b')],
            positional: vec![],
        })?;
        assert_eq!(result, vec!["--foo", "-a", "42", "-b"]);

        let result = equals_grammar.generate_vec(ParsedArgs {
            options: vec![short!('a', "a", "b", "c"), long!("foo", "1", "2")],
            positional: sv!["x"],
        })?;
        assert_eq!(result, vec!["-a", "a", "b", "c", "--foo", "1", "2", "--", "x"]);

        Ok(())
    }

    #[test]
    fn test_generate_not_explicit() -> Result<()> {
        let grammar = Gnu { explicit_positional: false, ..Default::default() };

        let result = grammar.generate_vec(ParsedArgs {
            options: vec![long!("long1", "42"), short!('s')],
            positional: sv!["pos1", "pos2"],
        })?;
        assert_eq!(result, vec!["--long1=42", "-s", "pos1", "pos2"]);

        let result = grammar.generate_vec(ParsedArgs {
            options: vec![short!('a', "a", "b", "c"), long!("foo", "1", "2")],
            positional: sv!["x"],
        })?;
        assert_eq!(result, vec!["-a", "a", "b", "c", "--foo=1", "2", "x"]);

        Ok(())
    }

    #[test]
    fn test_generate_edge_cases() -> Result<()> {
        let grammar = Gnu::default();

        // Empty arguments
        let result = grammar.generate_vec(ParsedArgs::default())?;
        assert!(result.is_empty());

        // Just positional arguments
        let result = grammar.generate_vec(ParsedArgs {
            positional: sv!["Hello", "New World!"],
            ..Default::default()
        })?;
        assert_eq!(result, vec!["--", "Hello", "New World!"]);

        // 1-character long options (non-standard in GNU)
        let result = grammar.generate_vec(ParsedArgs {
            options: vec![short!('z'), long!("z"), short!('z')],
            ..Default::default()
        })?;
        assert_eq!(result, vec!["-z", "--z", "-z"]);

        Ok(())
    }

    trait EasyParse {
        fn parse_arr(&self, args: &[&str]) -> Result<ParsedArgs>;
    }

    impl<G: Grammar> EasyParse for G {
        fn parse_arr(&self, args: &[&str]) -> Result<ParsedArgs> {
            self.parse(args.iter().map(|s| s.to_string()))
        }
    }
}

