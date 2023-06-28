macro_rules! assert_err {
        ($result:expr) => {
            let result = $result;
            if let Ok(_) = result {
                panic!("assertion failed:\nExpected error, got {:?}", result);
            }
        };
    }
macro_rules! assert_err_contains {
        ($result:expr, $msg:literal) => {
            let result = $result;
            match result {
                Ok(_) => {
                    panic!("assertion failed:\nExpected error containing: {}\nGot {:?}", $msg, result);
                },
                Err(e) => {
                    let msg = e.to_string();
                    if !msg.contains($msg) {
                        panic!("assertion failed:\nExpected error containing: {}\nGot error: {}", $msg, msg);
                    }
                }
            }
        };
    }

pub(crate) use assert_err;
pub(crate) use assert_err_contains;
