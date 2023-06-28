use std::borrow::Cow;
use derivative::Derivative;
use eyre::eyre;

#[derive(Clone, Debug, Derivative, Eq, PartialEq)]
#[derivative(Default)]
pub struct ParsedArgs {
    pub options: Vec<ParsedOpt>,
    pub positional: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedOpt {
    pub name: OptName,
    pub values: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptName {
    Long(String),
    Short(char),
}

impl ParsedArgs {
    pub fn remove_all_options(&mut self, name: OptName) {
        self.options.retain(|o| o.name != name);
    }

    pub fn update_last_option<F>(&mut self, name: OptName, update_values: F)
        where F: FnOnce(&mut Vec<String>)
    {
        match self.options.iter_mut().rfind(|o| o.name == name) {
            Some(option) => update_values(&mut option.values),
            None => {
                let mut values = vec![];
                update_values(&mut values);
                self.options.push(ParsedOpt { name, values })
            }
        }
    }

    pub fn set_last_option(&mut self, name: OptName, values: Vec<String>) {
        match self.options.iter_mut().rfind(|o| o.name == name) {
            Some(option) => option.values = values,
            None => self.options.push(ParsedOpt { name, values })
        }
    }
}

impl<'a> TryFrom<Cow<'a, str>> for OptName {
    type Error = eyre::Error;

    fn try_from(value: Cow<'a, str>) -> Result<Self, Self::Error> {
        match value {
            Cow::Borrowed(value) => Self::try_from(value),
            Cow::Owned(value) => Self::try_from(value),
        }
    }
}

impl TryFrom<&str> for OptName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.len() {
            0 => Err(eyre!("Option name cannot be empty")),
            1 => Ok(OptName::Short(value.chars().next().unwrap())),
            _ => Ok(OptName::Long(value.to_string())),
        }
    }
}

impl TryFrom<String> for OptName {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.len() {
            0 => Err(eyre!("Option name cannot be empty")),
            1 => Ok(OptName::Short(value.chars().next().unwrap())),
            _ => Ok(OptName::Long(value)),
        }
    }
}
