use std::convert::Infallible;

use crate::{
    error::{AnyOfError, EqualsError, NoneOfError, NotEqualsError},
    parser::{Done, Parser, Transition},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Any;

impl<I> Parser<I> for Any {
    type Output = I;
    type Error = Infallible;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
        Ok(Done(input))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Equals<T>(pub T);

impl<T, I> Parser<I> for Equals<T>
where
    T: PartialEq<I>,
{
    type Output = I;
    type Error = EqualsError<T, I>;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
        if self.0 == input {
            Ok(Done(input))
        } else {
            Err(EqualsError { expected: self.0, found: input })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NotEquals<T>(pub T);

impl<T, I> Parser<I> for NotEquals<T>
where
    T: PartialEq<I>,
{
    type Output = I;
    type Error = NotEqualsError<T, I>;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
        if self.0 == input {
            Err(NotEqualsError { unexpected: self.0, found: input })
        } else {
            Ok(Done(input))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AnyOf<It>(pub It)
where
    It: IntoIterator,
    It::IntoIter: Clone;

impl<It, I> Parser<I> for AnyOf<It>
where
    It: IntoIterator,
    It::IntoIter: Clone,
    It::Item: PartialEq<I>,
{
    type Output = I;
    type Error = AnyOfError<It::IntoIter, I>;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
        let iterator = self.0.into_iter();
        if iterator.clone().any(|elem| elem == input) {
            Ok(Done(input))
        } else {
            Err(AnyOfError { expecteds: iterator, found: input })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NoneOf<It>(pub It)
where
    It: IntoIterator,
    It::IntoIter: Clone;

impl<It, I> Parser<I> for NoneOf<It>
where
    It: IntoIterator,
    It::IntoIter: Clone,
    It::Item: PartialEq<I>,
{
    type Output = I;
    type Error = NoneOfError<It::IntoIter, I>;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
        let iterator = self.0.into_iter();
        if iterator.clone().all(|elem| elem != input) {
            Ok(Done(input))
        } else {
            Err(NoneOfError { unexpecteds: iterator, found: input })
        }
    }
}
