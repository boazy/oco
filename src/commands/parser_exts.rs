use pest::RuleType;
use pest::iterators::{Pair, Pairs};
use std::borrow::Cow;
use eyre::{bail, ContextCompat, Result};
use itertools::Itertools;
use tap::TapOptional;

pub trait SupportsQuoting {
    fn quoted_rule() -> Self;
}

pub trait PairExt<'i, R> {
    fn parse_quoted(self) -> Result<Cow<'i, str>>;
    fn parse_quoted_into_string(self) -> Result<String>;
}

impl<'i, R: RuleType + SupportsQuoting> PairExt<'i, R> for Pair<'i, R> {
    fn parse_quoted(self) -> Result<Cow<'i, str>> {
        if self.as_rule() != R::quoted_rule() {
            return Ok(self.as_str().into());
        };

        let str = self.as_str();
        let first_char = str.chars().next()
                .wrap_err("Empty quoted string")?;

        // Single-quoted strings: do not support escaping
        if first_char == '\'' && str.ends_with('\'') {
            return Ok((&str[1..str.len() - 1]).into());
        };

        // Single-quoted strings: support escaping
        if first_char == '"' && str.ends_with('"') {
            return Ok(str[1..str.len() - 1].unescape().into());
        };

        bail!("Invalid quoted string format: {}", self.as_str());
    }

    fn parse_quoted_into_string(self) -> Result<String> {
        self.parse_quoted().map(Cow::into_owned)
    }
}

pub trait InnerUnwrap<M> {
    fn map_single_wrapped(self) -> M;
}

impl<'i, R: RuleType> InnerUnwrap<Option<Pair<'i, R>>> for Pair<'i, R>
{
    fn map_single_wrapped(self) -> Option<Pair<'i, R>> {
        self.into_inner().next()
    }
}

impl<'i, R: RuleType> InnerUnwrap<Option<Pair<'i, R>>> for Option<Pair<'i, R>>
{
    fn map_single_wrapped(self) -> Self {
        self.and_then(|pair| pair.into_inner().next())
    }
}

pub trait PairsExt<'i, R> {
    fn expect_rule(&mut self, rule: R) -> Result<Pair<'i, R>>;
    fn expect_rules(&mut self, rules: &[R]) -> Result<Pair<'i, R>>;
    fn attempt_rule(&mut self, rule: R) -> Option<Pair<'i, R>>;
    fn attempt_rules(&mut self, rules: &[R]) -> Option<Pair<'i, R>>;
}

impl<'i, R: RuleType> PairsExt<'i, R> for Pairs<'i, R> {
    fn expect_rule(&mut self, rule: R) -> Result<Pair<'i, R>> {
        match self.next() {
            None =>
                bail!("Expected {rule:?}, found none"),
            Some(found) if found.as_rule() == rule =>
                Ok(found),
            Some(found) =>
                bail!("Expected {rule:?}, found {:?}", found.as_rule())
        }
    }

    fn expect_rules(&mut self, rules: &[R]) -> Result<Pair<'i, R>> {
        match self.next() {
            None =>
                bail!("Expected {}, found none", format_rules(rules)),
            Some(found) if rules.contains(&found.as_rule()) =>
                Ok(found),
            Some(found) =>
                bail!("Expected {}, found {:?}", format_rules(rules), found.as_rule())
        }
    }

    fn attempt_rule(&mut self, rule: R) -> Option<Pair<'i, R>> {
        self.peek().filter(|x| x.as_rule() == rule)
                .tap_some(|_| { self.next(); })
    }

    fn attempt_rules(&mut self, rules: &[R]) -> Option<Pair<'i, R>> {
        self.peek().filter(|x| rules.contains(&x.as_rule()))
                .tap_some(|_| { self.next(); })
    }
}

fn format_rules<R: RuleType>(rules: &[R]) -> String {
    rules.iter().map(|rule| format!("{:?}", rule)).join("|")
}

trait StringExt {
    fn unescape(self) -> String;
}

impl StringExt for &str {
    fn unescape(self) -> String {
        let mut escaped = false;
        let mut result = String::with_capacity(self.len());
        for ch in self.chars() {
            if ch != '\\' {
                result.push(ch)
            } else if escaped {
                result.push(ch);
                escaped = false
            } else {
                escaped = true
            }
        }
        result
    }
}
