use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EqualsError<T, I> {
    pub expected: T,
    pub found: I,
}

impl<T, I> fmt::Display for EqualsError<T, I>
where
    T: fmt::Display,
    I: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "expected {}, found {}", self.expected, self.found)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotEqualsError<T, I> {
    pub unexpected: T,
    pub found: I,
}

impl<T, I> fmt::Display for NotEqualsError<T, I>
where
    T: fmt::Display,
    I: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} not allowed, found {}",
            self.unexpected, self.found
        )
    }
}
