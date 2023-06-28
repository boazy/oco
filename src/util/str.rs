pub trait SplitExt: Sized {
    fn split_kv(self) -> (Self, Option<Self>);
}

impl <'a> SplitExt for &'a str {
    fn split_kv(self) -> (Self, Option<Self>) {
        match self.split_once('=') {
            None => (self, None),
            Some((k, v)) => (k, Some(v))
        }
    }
}
