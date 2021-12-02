use std::{fmt, marker::PhantomData};

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
    type Input: Clone;
    type Output;
    type Error;

    fn transit(
        self,
        input: Self::Input,
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

    fn err_into<E>(self) -> ErrInto<Self, E>
    where
        E: From<Self::Error>,
    {
        ErrInto { inner: self, _marker: PhantomData }
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

    fn and<Q>(self, other: Q) -> And<Self, Q>
    where
        Q: Parser<Input = Self::Input, Error = Self::Error>,
    {
        And { left: Parsing(self), right: Parsing(other) }
    }

    fn then<Q>(self, second: Q) -> Then<Self, Q>
    where
        Q: Parser<Input = Self::Input, Error = Self::Error>,
    {
        Then { state: ThenState::ParseLeft { left: self, right: second } }
    }
}

#[derive(Debug, Clone, Copy)]
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
        input: Self::Input,
    ) -> Result<Transition<Self>, Self::Error> {
        match self.inner.transit(input.clone())? {
            Done(item) => Ok(Done((self.mapper)(item))),
            Parsing(inner) => Ok(Parsing(Self { inner, ..self })),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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
        input: Self::Input,
    ) -> Result<Transition<Self>, Self::Error> {
        match self.inner.transit(input.clone()) {
            Ok(Done(item)) => Ok(Done(item)),
            Ok(Parsing(inner)) => Ok(Parsing(Self { inner, ..self })),
            Err(error) => Err((self.mapper)(error)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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
        input: Self::Input,
    ) -> Result<Transition<Self>, Self::Error> {
        match self.inner.transit(input.clone()) {
            Ok(Done(item)) => (self.mapper)(Ok(item)).map(Done),
            Ok(Parsing(inner)) => Ok(Parsing(Self { inner, ..self })),
            Err(error) => (self.mapper)(Err(error)).map(Done),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ErrInto<P, E>
where
    P: Parser,
    E: From<P::Error>,
{
    inner: P,
    _marker: PhantomData<E>,
}

impl<P, E> Parser for ErrInto<P, E>
where
    P: Parser,
    E: From<P::Error>,
{
    type Input = P::Input;
    type Output = P::Output;
    type Error = E;

    fn transit(
        self,
        input: Self::Input,
    ) -> Result<Transition<Self>, Self::Error> {
        match self.inner.transit(input.clone()) {
            Ok(Done(item)) => Ok(Done(item)),
            Ok(Parsing(inner)) => Ok(Parsing(Self { inner, ..self })),
            Err(error) => Err(E::from(error)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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
        input: Self::Input,
    ) -> Result<Transition<Self>, Self::Error> {
        match self.left.transit(input.clone())? {
            Done(item) => Ok(Done(item)),
            Parsing(left) => match self.right.transit(input)? {
                Done(item) => Ok(Done(item)),
                Parsing(right) => Ok(Parsing(Self { left, right })),
            },
        }
    }
}

pub struct And<P, Q>
where
    P: Parser,
    Q: Parser<Input = P::Input, Error = P::Error>,
{
    left: Transition<P>,
    right: Transition<Q>,
}

impl<P, Q> Parser for And<P, Q>
where
    P: Parser,
    Q: Parser<Input = P::Input, Error = P::Error>,
{
    type Input = P::Input;
    type Output = (P::Output, Q::Output);
    type Error = P::Error;

    fn transit(
        self,
        input: Self::Input,
    ) -> Result<Transition<Self>, Self::Error> {
        let left = match self.left {
            Parsing(parser) => parser.transit(input.clone())?,
            Done(item) => Done(item),
        };

        let right = match self.right {
            Parsing(parser) => parser.transit(input)?,
            Done(item) => Done(item),
        };

        match (left, right) {
            (Done(first), Done(second)) => Ok(Done((first, second))),
            (left, right) => Ok(Parsing(Self { left, right })),
        }
    }
}

impl<P, Q> fmt::Debug for And<P, Q>
where
    P: Parser + fmt::Debug,
    Q: Parser<Input = P::Input, Error = P::Error> + fmt::Debug,
    P::Output: fmt::Debug,
    Q::Output: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("And")
            .field("left", &self.left)
            .field("right", &self.right)
            .finish()
    }
}

impl<P, Q> Clone for And<P, Q>
where
    P: Parser + Clone,
    Q: Parser<Input = P::Input, Error = P::Error> + Clone,
    P::Output: Clone,
    Q::Output: Clone,
{
    fn clone(&self) -> Self {
        Self { left: self.left.clone(), right: self.right.clone() }
    }
}

impl<P, Q> Copy for And<P, Q>
where
    P: Parser + Copy,
    Q: Parser<Input = P::Input, Error = P::Error> + Copy,
    P::Output: Copy,
    Q::Output: Copy,
{
}

#[derive(Debug, Clone, Copy)]
enum ThenState<P, Q>
where
    P: Parser,
    Q: Parser<Input = P::Input, Error = P::Error>,
{
    ParseLeft { left: P, right: Q },
    ParseRight { left_output: P::Output, right: Q },
}

pub struct Then<P, Q>
where
    P: Parser,
    Q: Parser<Input = P::Input, Error = P::Error>,
{
    state: ThenState<P, Q>,
}

impl<P, Q> Parser for Then<P, Q>
where
    P: Parser,
    Q: Parser<Input = P::Input, Error = P::Error>,
{
    type Input = P::Input;
    type Output = (P::Output, Q::Output);
    type Error = P::Error;

    fn transit(
        self,
        input: Self::Input,
    ) -> Result<Transition<Self>, Self::Error> {
        match self.state {
            ThenState::ParseLeft { left, right } => {
                match left.transit(input.clone())? {
                    Parsing(left) => Ok(Parsing(Self {
                        state: ThenState::ParseLeft { left, right },
                    })),

                    Done(item) => Ok(Parsing(Self {
                        state: ThenState::ParseRight {
                            left_output: item,
                            right,
                        },
                    })),
                }
            },

            ThenState::ParseRight { left_output, right } => {
                match right.transit(input)? {
                    Parsing(right) => Ok(Parsing(Self {
                        state: ThenState::ParseRight { left_output, right },
                    })),

                    Done(item) => Ok(Done((left_output, item))),
                }
            },
        }
    }
}

impl<P, Q> fmt::Debug for Then<P, Q>
where
    P: Parser + fmt::Debug,
    Q: Parser<Input = P::Input, Error = P::Error> + fmt::Debug,
    P::Output: fmt::Debug,
    Q::Output: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Then").field("state", &self.state).finish()
    }
}

impl<P, Q> Clone for Then<P, Q>
where
    P: Parser + Clone,
    Q: Parser<Input = P::Input, Error = P::Error> + Clone,
    P::Output: Clone,
    Q::Output: Clone,
{
    fn clone(&self) -> Self {
        Self { state: self.state.clone() }
    }
}

impl<P, Q> Copy for Then<P, Q>
where
    P: Parser + Copy,
    Q: Parser<Input = P::Input, Error = P::Error> + Copy,
    P::Output: Copy,
    Q::Output: Copy,
{
}
