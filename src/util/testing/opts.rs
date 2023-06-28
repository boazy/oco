// Create Vec<String>() from literals
macro_rules! sv {
        ($($arg:literal),*) => { vec![$($arg.to_string()),*] }
    }

pub mod parsed {
    macro_rules! short {
        ($name:literal $(,)? $($arg:literal),*) => {
            crate::opts::parsed_args::ParsedOpt {
                name: crate::util::testing::opts::name::short!($name),
                values: sv![$($arg),*],
            }
        }
    }

    macro_rules! long {
        ($name:literal $(,)? $($arg:literal),*) => {
            crate::opts::parsed_args::ParsedOpt {
                name: crate::util::testing::opts::name::long!($name),
                values: sv![$($arg),*],
            }
        }
    }

    pub(crate) use short;
    pub(crate) use long;
}

pub mod name {
    macro_rules! short {
        ($name:literal $(,)? $($arg:literal),*) => {
            crate::opts::parsed_args::OptName::Short($name)
        }
    }

    macro_rules! long {
        ($name:literal $(,)? $($arg:literal),*) => {
                crate::opts::parsed_args::OptName::Long($name.to_string())
        }
    }

    pub(crate) use short;
    pub(crate) use long;
}

pub(crate) use sv;
