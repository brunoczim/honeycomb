use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub enum GeneralError<T, I, It> {
    Equals(EqualsError<T, I>),
    NotEquals(NotEqualsError<T, I>),
    AnyOf(AnyOfError<It, I>),
    NoneOf(NoneOfError<It, I>),
    UnexpectedEndOfInput(UnexpectedEndOfInput),
    ExpectedEndOfInput(ExpectedEndOfInput<I>),
    Digit(DigitError<I>),
}

impl<T, I, It> fmt::Display for GeneralError<T, I, It>
where
    T: fmt::Display,
    I: fmt::Display,
    It: Iterator + Clone,
    It::Item: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GeneralError::Equals(error) => write!(formatter, "{}", error),
            GeneralError::NotEquals(error) => write!(formatter, "{}", error),
            GeneralError::AnyOf(error) => write!(formatter, "{}", error),
            GeneralError::NoneOf(error) => write!(formatter, "{}", error),
            GeneralError::UnexpectedEndOfInput(error) => {
                write!(formatter, "{}", error)
            },
            GeneralError::ExpectedEndOfInput(error) => {
                write!(formatter, "{}", error)
            },
            GeneralError::Digit(error) => write!(formatter, "{}", error),
        }
    }
}

impl<T, I, It> Error for GeneralError<T, I, It>
where
    T: fmt::Display + fmt::Debug,
    I: fmt::Display + fmt::Debug,
    It: Iterator + Clone + fmt::Debug,
    It::Item: fmt::Display,
{
}

impl<T, I, It> From<EqualsError<T, I>> for GeneralError<T, I, It> {
    fn from(error: EqualsError<T, I>) -> Self {
        GeneralError::Equals(error)
    }
}

impl<T, I, It> From<NotEqualsError<T, I>> for GeneralError<T, I, It> {
    fn from(error: NotEqualsError<T, I>) -> Self {
        GeneralError::NotEquals(error)
    }
}

impl<T, I, It> From<AnyOfError<It, I>> for GeneralError<T, I, It> {
    fn from(error: AnyOfError<It, I>) -> Self {
        GeneralError::AnyOf(error)
    }
}

impl<T, I, It> From<NoneOfError<It, I>> for GeneralError<T, I, It> {
    fn from(error: NoneOfError<It, I>) -> Self {
        GeneralError::NoneOf(error)
    }
}

impl<T, I, It> From<UnexpectedEndOfInput> for GeneralError<T, I, It> {
    fn from(error: UnexpectedEndOfInput) -> Self {
        GeneralError::UnexpectedEndOfInput(error)
    }
}

impl<T, I, It> From<ExpectedEndOfInput<I>> for GeneralError<T, I, It> {
    fn from(error: ExpectedEndOfInput<I>) -> Self {
        GeneralError::ExpectedEndOfInput(error)
    }
}

impl<T, I, It> From<DigitError<I>> for GeneralError<T, I, It> {
    fn from(error: DigitError<I>) -> Self {
        GeneralError::Digit(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EqualsError<T, I> {
    pub expected: T,
    pub found: I,
}

impl<T, I> Error for EqualsError<T, I>
where
    T: fmt::Display + fmt::Debug,
    I: fmt::Display + fmt::Debug,
{
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

impl<T, I> Error for NotEqualsError<T, I>
where
    T: fmt::Display + fmt::Debug,
    I: fmt::Display + fmt::Debug,
{
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnyOfError<It, I> {
    pub expecteds: It,
    pub found: I,
}

impl<It, I> Error for AnyOfError<It, I>
where
    It: Iterator + Clone + fmt::Debug,
    It::Item: fmt::Display,
    I: fmt::Display + fmt::Debug,
{
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoneOfError<It, I> {
    pub unexpecteds: It,
    pub found: I,
}

impl<It, I> Error for NoneOfError<It, I>
where
    It: Iterator + Clone + fmt::Debug,
    It::Item: fmt::Display,
    I: fmt::Display + fmt::Debug,
{
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnexpectedEndOfInput;

impl Error for UnexpectedEndOfInput {}

impl fmt::Display for UnexpectedEndOfInput {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "unexpected end of input")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl<I> Error for ExpectedEndOfInput<I> where I: fmt::Display + fmt::Debug {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DigitError<I> {
    pub base: u8,
    pub found: I,
}

impl<I> fmt::Display for DigitError<I>
where
    I: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "expected digit of base {}, found {}",
            self.base, self.found
        )
    }
}

impl<I> Error for DigitError<I> where I: fmt::Display + fmt::Debug {}
