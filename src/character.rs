use crate::{
    error::DigitError,
    parser::{Done, Parser, Transition},
};

pub trait AsChar {
    fn as_char(&self) -> Option<char>;
}

impl AsChar for char {
    fn as_char(&self) -> Option<char> {
        Some(*self)
    }
}

impl AsChar for u8 {
    fn as_char(&self) -> Option<char> {
        Some(char::from(*self))
    }
}

impl<'this, T> AsChar for &'this T
where
    T: AsChar + ?Sized,
{
    fn as_char(&self) -> Option<char> {
        (**self).as_char()
    }
}

impl<T> AsChar for Option<T>
where
    T: AsChar,
{
    fn as_char(&self) -> Option<char> {
        self.as_ref().and_then(AsChar::as_char)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Digit {
    base: u32,
}

impl Digit {
    pub fn new(base: u32) -> Self {
        Self { base }
    }
}

impl<I> Parser<I> for Digit
where
    I: AsChar,
{
    type Output = u32;
    type Error = DigitError<I>;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
        let maybe_ch = input.as_char();
        match maybe_ch.and_then(|ch| ch.to_digit(self.base)) {
            Some(value) => Ok(Done(value)),
            None => Err(DigitError { base: self.base, found: input }),
        }
    }
}
