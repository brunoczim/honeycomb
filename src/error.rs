use std::fmt;

struct DisplayOrList<'it, It>(&'it It);

impl<'it, It> fmt::Display for DisplayOrList<'it, It>
where
    It: Iterator + Clone,
    It::Item: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut iterator = self.0.clone();
        match iterator.next() {
            Some(elem) => {
                write!(formatter, "{}", elem)?;
                if let Some(mut current) = iterator.next() {
                    for next in iterator.next() {
                        write!(formatter, ", {}", current)?;
                        current = next;
                    }
                    write!(formatter, " or {}", current)?;
                }
                Ok(())
            },
            None => write!(formatter, "nothing"),
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyOfError<It, I> {
    pub expecteds: It,
    pub found: I,
}

impl<It, I> fmt::Display for AnyOfError<It, I>
where
    It: Iterator + Clone,
    It::Item: fmt::Display,
    I: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "expected {}, found {}",
            DisplayOrList(&self.expecteds),
            self.found
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoneOfError<It, I> {
    pub unexpecteds: It,
    pub found: I,
}

impl<It, I> fmt::Display for NoneOfError<It, I>
where
    It: Iterator + Clone,
    It::Item: fmt::Display,
    I: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} not allowed, found {}",
            DisplayOrList(&self.unexpecteds),
            self.found
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnexpectedEndOfInput;

impl fmt::Display for UnexpectedEndOfInput {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "unexpected end of input")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedEndOfInput<I> {
    pub found: I,
}

impl<I> fmt::Display for ExpectedEndOfInput<I>
where
    I: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "expected end of input, found {}", self.found)
    }
}
