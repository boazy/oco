pub trait PushExt<T> {
    fn ensure_first_or_default(&mut self) -> &mut T
        where T: Default;
    fn push_and_borrow(&mut self, value: T) -> &mut T;
}

impl<T> PushExt<T> for Vec<T> {
    fn ensure_first_or_default(&mut self) -> &mut T where T: Default {
        if self.is_empty() {
            self.push(Default::default());
        }
        self.first_mut().unwrap()
    }

    fn push_and_borrow(&mut self, value: T) -> &mut T {
        self.push(value);
        self.last_mut().unwrap()
    }
}