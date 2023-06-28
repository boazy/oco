use std::io::{BufRead};




pub fn read_args<B: BufRead>(b: B) -> impl Iterator<Item=String> {
    b.lines().flat_map(|l| {
        let l = l.expect("Cannot read input");
        shlex::split(l.as_str()).expect("Bad command line input")
    })
}
