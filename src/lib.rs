#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transition<P>
where
    P: Parser,
{
    Parsing(P),
    Done(P::Output),
}

pub use Transition::*;

pub trait Parser: Sized {
    type Input: ?Sized;
    type Output;
    type Error;

    fn transit(
        self,
        input: &Self::Input,
    ) -> Result<Transition<Self>, Self::Error>;

    fn map<F, T>(self, mapper: F) -> Map<Self, F, T>
    where
        F: FnOnce(Self::Output) -> T,
    {
        Map { inner: self, mapper }
    }

    fn map_err<F, E>(self, mapper: F) -> MapErr<Self, F, E>
    where
        F: FnOnce(Self::Error) -> E,
    {
        MapErr { inner: self, mapper }
    }

    fn map_res<F, T, E>(self, mapper: F) -> MapResult<Self, F, T, E>
    where
        F: FnOnce(Result<Self::Output, Self::Error>) -> Result<T, E>,
    {
        MapResult { inner: self, mapper }
    }

    fn or<Q>(self, other: Q) -> Or<Self, Q>
    where
        Q: Parser<
            Input = Self::Input,
            Output = Self::Output,
            Error = Self::Error,
        >,
    {
        Or { left: self, right: other }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Map<P, F, T>
where
    P: Parser,
    F: FnOnce(P::Output) -> T,
{
    inner: P,
    mapper: F,
}

impl<P, F, T> Parser for Map<P, F, T>
where
    P: Parser,
    F: FnOnce(P::Output) -> T,
{
    type Input = P::Input;
    type Output = T;
    type Error = P::Error;

    fn transit(
        self,
        input: &Self::Input,
    ) -> Result<Transition<Self>, Self::Error> {
        match self.inner.transit(input)? {
            Done(item) => Ok(Done((self.mapper)(item))),
            Parsing(inner) => Ok(Parsing(Self { inner, ..self })),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MapErr<P, F, E>
where
    P: Parser,
    F: FnOnce(P::Error) -> E,
{
    inner: P,
    mapper: F,
}

impl<P, F, E> Parser for MapErr<P, F, E>
where
    P: Parser,
    F: FnOnce(P::Error) -> E,
{
    type Input = P::Input;
    type Output = P::Output;
    type Error = E;

    fn transit(
        self,
        input: &Self::Input,
    ) -> Result<Transition<Self>, Self::Error> {
        match self.inner.transit(input) {
            Ok(Done(item)) => Ok(Done(item)),
            Ok(Parsing(inner)) => Ok(Parsing(Self { inner, ..self })),
            Err(error) => Err((self.mapper)(error)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MapResult<P, F, T, E>
where
    P: Parser,
    F: FnOnce(Result<P::Output, P::Error>) -> Result<T, E>,
{
    inner: P,
    mapper: F,
}

impl<P, F, T, E> Parser for MapResult<P, F, T, E>
where
    P: Parser,
    F: FnOnce(Result<P::Output, P::Error>) -> Result<T, E>,
{
    type Input = P::Input;
    type Output = T;
    type Error = E;

    fn transit(
        self,
        input: &Self::Input,
    ) -> Result<Transition<Self>, Self::Error> {
        match self.inner.transit(input) {
            Ok(Done(item)) => (self.mapper)(Ok(item)).map(Done),
            Ok(Parsing(inner)) => Ok(Parsing(Self { inner, ..self })),
            Err(error) => (self.mapper)(Err(error)).map(Done),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Or<P, Q>
where
    P: Parser,
    Q: Parser<Input = P::Input, Output = P::Output, Error = P::Error>,
{
    left: P,
    right: Q,
}

impl<P, Q> Parser for Or<P, Q>
where
    P: Parser,
    Q: Parser<Input = P::Input, Output = P::Output, Error = P::Error>,
{
    type Input = P::Input;
    type Output = P::Output;
    type Error = P::Error;

    fn transit(
        self,
        input: &Self::Input,
    ) -> Result<Transition<Self>, Self::Error> {
        match self.left.transit(input)? {
            Done(item) => Ok(Done(item)),
            Parsing(left) => match self.right.transit(input)? {
                Done(item) => Ok(Done(item)),
                Parsing(right) => Ok(Parsing(Self { left, right })),
            },
        }
    }
}
