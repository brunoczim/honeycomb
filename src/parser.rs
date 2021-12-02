use crate::combinator::{
    And,
    ErrInto,
    FilterMap,
    Map,
    MapErr,
    MapInput,
    Or,
    Then,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transition<P, T> {
    Parsing(P),
    Done(T),
}

pub use Transition::*;

pub trait Parser<I> {
    type Output;
    type Error;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error>
    where
        Self: Sized;

    fn map<F, T>(self, mapper: F) -> Map<Self, F>
    where
        I: Clone,
        F: FnOnce(Self::Output) -> T,
        Self: Sized,
    {
        Map::new(self, mapper)
    }

    fn map_err<F, E>(self, mapper: F) -> MapErr<Self, F>
    where
        I: Clone,
        F: FnOnce(Self::Error) -> E,
        Self: Sized,
    {
        MapErr::new(self, mapper)
    }

    fn filter_map<F, T, E>(self, mapper: F) -> FilterMap<Self, F>
    where
        I: Clone,
        F: FnOnce(Result<Self::Output, Self::Error>) -> Result<T, E>,
        Self: Sized,
    {
        FilterMap::new(self, mapper)
    }

    fn err_into<E>(self) -> ErrInto<Self, E>
    where
        I: Clone,
        E: From<Self::Error>,
        Self: Sized,
    {
        ErrInto::new(self)
    }

    fn map_input<F, J>(self, mapper: F) -> MapInput<Self, F>
    where
        F: FnMut(J) -> I,
        Self: Sized,
    {
        MapInput::new(self, mapper)
    }

    fn or<Q>(self, other: Q) -> Or<Self, Q>
    where
        I: Clone,
        Q: Parser<I, Output = Self::Output, Error = Self::Error>,
        Self: Sized,
    {
        Or::new(self, other)
    }

    fn and<Q, T, U>(self, other: Q) -> And<Self, Q, Self::Output, Q::Output>
    where
        I: Clone,
        Q: Parser<I, Error = Self::Error>,
        Self: Sized,
    {
        And::new(self, other)
    }

    fn then<Q>(self, second: Q) -> Then<Self, Q, Self::Output>
    where
        I: Clone,
        Q: Parser<I, Error = Self::Error>,
        Self: Sized,
    {
        Then::new(self, second)
    }
}

impl<T, I> Parser<I> for Box<T>
where
    T: Parser<I>,
{
    type Output = T::Output;
    type Error = T::Error;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error> {
        match (*self).transit(input)? {
            Done(item) => Ok(Done(item)),
            Parsing(parser) => Ok(Parsing(Box::new(parser))),
        }
    }
}

pub trait BoxedParser<'obj, I>: Parser<I> {
    fn transit_boxed(
        self: Box<Self>,
        input: I,
    ) -> Result<
        Transition<DynParser<'obj, I, Self::Output, Self::Error>, Self::Output>,
        Self::Error,
    >;

    fn dyn_clone(&self) -> DynParser<'obj, I, Self::Output, Self::Error>;
}

impl<'obj, T, I> BoxedParser<'obj, I> for T
where
    T: Parser<I> + 'obj + Send + Sync + Clone,
{
    fn transit_boxed(
        self: Box<Self>,
        input: I,
    ) -> Result<
        Transition<DynParser<'obj, I, Self::Output, Self::Error>, Self::Output>,
        Self::Error,
    > {
        match (*self).transit(input)? {
            Done(item) => Ok(Done(item)),
            Parsing(parser) => Ok(Parsing(Box::new(parser))),
        }
    }

    fn dyn_clone(&self) -> DynParser<'obj, I, Self::Output, Self::Error> {
        Box::new(self.clone())
    }
}

pub type DynParser<'obj, I, O, E> =
    Box<dyn BoxedParser<'obj, I, Output = O, Error = E> + 'obj + Send + Sync>;

impl<'obj, I, O, E> Parser<I> for DynParser<'obj, I, O, E> {
    type Output = O;
    type Error = E;

    fn transit(
        self,
        input: I,
    ) -> Result<Transition<Self, Self::Output>, Self::Error>
    where
        Self: Sized,
    {
        <dyn BoxedParser<I, Output = O, Error = E>>::transit_boxed(self, input)
    }
}

impl<'obj, I, O, E> Clone for DynParser<'obj, I, O, E>
where
    I: 'obj,
    O: 'obj,
    E: 'obj,
{
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}
