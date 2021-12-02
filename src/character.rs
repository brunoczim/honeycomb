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
    pub base: u8,
}

impl<I> Parser<I> for Digit
where
    I: AsChar,
{
    type Output = u8;
    type Error = DigitError<I>;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
        let maybe_ch = input.as_char();
        match maybe_ch.and_then(|ch| ch.to_digit(u32::from(self.base))) {
            Some(value) => Ok(Done(value as u8)),
            None => Err(DigitError { base: self.base, found: input }),
        }
    }
}
