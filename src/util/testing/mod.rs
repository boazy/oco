mod assert_errs;
pub mod opts;

pub(crate) use assert_errs::assert_err;
pub(crate) use assert_errs::assert_err_contains;

#[cfg(test)]
#[ctor::ctor]
fn common_test_init() {
    color_eyre::install().expect("Failed to install `color_eyre`");
}