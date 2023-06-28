pub trait ResultIter<I, T, E>
{
    fn and_then<F, U>(self, f: F) -> AndThen<I, F>
        where F: FnMut(T) -> Result<U, E>;
}

impl<I, T, E> ResultIter<I, T, E> for I
    where I: Iterator<Item=Result<T, E>>
{
    fn and_then<F, U>(self, f: F) -> AndThen<I, F>
        where F: FnMut(T) -> Result<U, E>
    {
        AndThen { iter: self, f }
    }
}

pub struct AndThen<I, F> {
    iter: I,
    f: F,
}

impl<T, E, U, I, F> Iterator for AndThen<I, F>
    where
            I: Iterator<Item=Result<T, E>>,
            F: FnMut(T) -> Result<U, E>,
{
    type Item = Result<U, E>;

    #[inline]
    fn next(&mut self) -> Option<Result<U, E>> {
        self.iter
                .next()
                .map(|result|
                        result.and_then(|value| (self.f)(value))
                )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
