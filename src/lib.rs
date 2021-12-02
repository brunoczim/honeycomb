use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transition<P, T> {
    Parsing(P),
    Done(T),
}

pub use Transition::*;

pub trait Parser<I>: Sized
where
    I: Clone,
{
    type Output;
    type Error;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error>;

    fn map<F, T>(self, mapper: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Output) -> T,
    {
        Map { inner: self, mapper }
    }

    fn map_err<F, E>(self, mapper: F) -> MapErr<Self, F>
    where
        F: FnOnce(Self::Error) -> E,
    {
        MapErr { inner: self, mapper }
    }

    fn map_res<F, T, E>(self, mapper: F) -> MapResult<Self, F>
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
        Q: Parser<I, Output = Self::Output, Error = Self::Error>,
    {
        Or { left: self, right: other }
    }

    fn and<Q, T, U>(self, other: Q) -> And<Self, Q, Self::Output, Q::Output>
    where
        Q: Parser<I, Error = Self::Error>,
    {
        And { left: Parsing(self), right: Parsing(other) }
    }

    fn then<Q>(self, second: Q) -> Then<Self, Q, Self::Output>
    where
        Q: Parser<I, Error = Self::Error>,
    {
        Then { state: ThenState::ParseLeft { left: self, right: second } }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Map<P, F> {
    inner: P,
    mapper: F,
}

impl<P, F, I, T> Parser<I> for Map<P, F>
where
    I: Clone,
    P: Parser<I>,
    F: FnOnce(P::Output) -> T,
{
    type Output = T;
    type Error = P::Error;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
        match self.inner.transit(input.clone())? {
            Done(item) => Ok(Done((self.mapper)(item))),
            Parsing(inner) => Ok(Parsing(Self { inner, ..self })),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MapErr<P, F> {
    inner: P,
    mapper: F,
}

impl<P, F, I, E> Parser<I> for MapErr<P, F>
where
    I: Clone,
    P: Parser<I>,
    F: FnOnce(P::Error) -> E,
{
    type Output = P::Output;
    type Error = E;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
        match self.inner.transit(input.clone()) {
            Ok(Done(item)) => Ok(Done(item)),
            Ok(Parsing(inner)) => Ok(Parsing(Self { inner, ..self })),
            Err(error) => Err((self.mapper)(error)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MapResult<P, F> {
    inner: P,
    mapper: F,
}

impl<P, F, I, T, E> Parser<I> for MapResult<P, F>
where
    I: Clone,
    P: Parser<I>,
    F: FnOnce(Result<P::Output, P::Error>) -> Result<T, E>,
{
    type Output = T;
    type Error = E;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
        match self.inner.transit(input.clone()) {
            Ok(Done(item)) => (self.mapper)(Ok(item)).map(Done),
            Ok(Parsing(inner)) => Ok(Parsing(Self { inner, ..self })),
            Err(error) => (self.mapper)(Err(error)).map(Done),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ErrInto<P, E> {
    inner: P,
    _marker: PhantomData<E>,
}

impl<P, E, I> Parser<I> for ErrInto<P, E>
where
    I: Clone,
    P: Parser<I>,
    E: From<P::Error>,
{
    type Output = P::Output;
    type Error = E;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
        match self.inner.transit(input.clone()) {
            Ok(Done(item)) => Ok(Done(item)),
            Ok(Parsing(inner)) => Ok(Parsing(Self { inner, ..self })),
            Err(error) => Err(E::from(error)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Or<P, Q> {
    left: P,
    right: Q,
}

impl<P, Q, I> Parser<I> for Or<P, Q>
where
    I: Clone,
    P: Parser<I>,
    Q: Parser<I, Output = P::Output, Error = P::Error>,
{
    type Output = P::Output;
    type Error = P::Error;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
        match self.left.transit(input.clone())? {
            Done(item) => Ok(Done(item)),
            Parsing(left) => match self.right.transit(input)? {
                Done(item) => Ok(Done(item)),
                Parsing(right) => Ok(Parsing(Self { left, right })),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct And<P, Q, T, U> {
    left: Transition<P, T>,
    right: Transition<Q, U>,
}

impl<P, Q, T, U, I> Parser<I> for And<P, Q, T, U>
where
    I: Clone,
    P: Parser<I, Output = T>,
    Q: Parser<I, Error = P::Error, Output = U>,
{
    type Output = (P::Output, Q::Output);
    type Error = P::Error;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
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

#[derive(Debug, Clone, Copy)]
enum ThenState<P, Q, T> {
    ParseLeft { left: P, right: Q },
    ParseRight { left_output: T, right: Q },
}

#[derive(Debug, Clone, Copy)]
pub struct Then<P, Q, T> {
    state: ThenState<P, Q, T>,
}

impl<P, Q, T, I> Parser<I> for Then<P, Q, T>
where
    I: Clone,
    P: Parser<I, Output = T>,
    Q: Parser<I, Error = P::Error>,
{
    type Output = (P::Output, Q::Output);
    type Error = P::Error;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
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
