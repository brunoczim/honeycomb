use crate::combinator::{
    ErrorInto,
    FatalInto,
    Map,
    MapError,
    MapFatal,
    MapInput,
    MapResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transition<P, T, E> {
    Done { output: T },
    Parsing { parser: P, error: Option<E> },
}

impl<P, T, E> Transition<P, T, E> {
    pub fn map_output<F, U>(self, mapper: F) -> Transition<P, U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Transition::Done { output } => {
                Transition::Done { output: mapper(output) }
            },
            Transition::Parsing { parser, error } => {
                Transition::Parsing { parser, error }
            },
        }
    }

    pub fn map_parser<F, Q>(self, mapper: F) -> Transition<Q, T, E>
    where
        F: FnOnce(P) -> Q,
    {
        match self {
            Transition::Done { output } => Transition::Done { output },
            Transition::Parsing { parser, error } => {
                Transition::Parsing { parser: mapper(parser), error }
            },
        }
    }

    pub fn map_error<F, E1>(self, mapper: F) -> Transition<P, T, E1>
    where
        F: FnOnce(E) -> E1,
    {
        match self {
            Transition::Done { output } => Transition::Done { output },
            Transition::Parsing { parser, error } => {
                Transition::Parsing { parser, error: error.map(mapper) }
            },
        }
    }
}

pub type TransitResult<P, I> = Result<
    Transition<P, <P as Parser<I>>::Output, <P as Parser<I>>::Error>,
    <P as Parser<I>>::Fatal,
>;

pub trait Parser<I> {
    type Output;
    type Error;
    type Fatal;

    fn transit(self, input: I) -> TransitResult<Self, I>
    where
        Self: Sized;

    fn map<F, T>(self, mapper: F) -> Map<Self, I, F, T>
    where
        Self: Sized,
        F: FnOnce(Self::Output) -> T,
    {
        Map::new(self, mapper)
    }

    fn map_input<F, J>(self, mapper: F) -> MapInput<Self, I, J, F>
    where
        Self: Sized,
        F: FnMut(J) -> I,
    {
        MapInput::new(self, mapper)
    }

    fn map_error<F, E>(self, mapper: F) -> MapError<Self, I, F, E>
    where
        Self: Sized,
        F: FnMut(Self::Error) -> E,
    {
        MapError::new(self, mapper)
    }

    fn map_fatal<F, Ef>(self, mapper: F) -> MapFatal<Self, I, F, Ef>
    where
        Self: Sized,
        F: FnOnce(Self::Fatal) -> Ef,
    {
        MapFatal::new(self, mapper)
    }

    fn map_result<F, T, Ef>(self, mapper: F) -> MapResult<Self, I, F, T, Ef>
    where
        Self: Sized,
        F: FnOnce(Result<Self::Output, Self::Fatal>) -> Result<T, Ef>,
    {
        MapResult::new(self, mapper)
    }

    fn error_into<E>(self) -> ErrorInto<Self, I, E>
    where
        Self: Sized,
        E: From<Self::Error>,
    {
        ErrorInto::new(self)
    }

    fn fatal_into<Ef>(self) -> FatalInto<Self, I, Ef>
    where
        Self: Sized,
        Ef: From<Self::Fatal>,
    {
        FatalInto::new(self)
    }
}

impl<T, I> Parser<I> for Box<T>
where
    T: Parser<I>,
{
    type Output = T::Output;
    type Error = T::Error;
    type Fatal = T::Fatal;

    fn transit(self, input: I) -> TransitResult<Self, I> {
        (*self).transit(input).map(|transition| transition.map_parser(Box::new))
    }
}

pub trait BoxedParser<'obj, I>: Parser<I> {
    fn transit_boxed(
        self: Box<Self>,
        input: I,
    ) -> TransitResult<
        DynParser<'obj, I, Self::Output, Self::Error, Self::Fatal>,
        I,
    >;

    fn dyn_clone(
        &self,
    ) -> DynParser<'obj, I, Self::Output, Self::Error, Self::Fatal>;
}

impl<'obj, T, I> BoxedParser<'obj, I> for T
where
    T: Parser<I> + 'obj + Send + Sync + Clone,
{
    fn transit_boxed(
        self: Box<Self>,
        input: I,
    ) -> TransitResult<
        DynParser<'obj, I, Self::Output, Self::Error, Self::Fatal>,
        I,
    > {
        (*self).transit(input).map(|transition| {
            transition.map_parser(|parser| Box::new(parser) as _)
        })
    }

    fn dyn_clone(
        &self,
    ) -> DynParser<'obj, I, Self::Output, Self::Error, Self::Fatal> {
        Box::new(self.clone())
    }
}

pub type DynParser<'obj, I, O, E, Ef> = Box<
    dyn BoxedParser<'obj, I, Output = O, Error = E, Fatal = Ef>
        + 'obj
        + Send
        + Sync,
>;

impl<'obj, I, O, E, Ef> Parser<I> for DynParser<'obj, I, O, E, Ef> {
    type Output = O;
    type Error = E;
    type Fatal = Ef;

    fn transit(self, input: I) -> TransitResult<Self, I> {
        self.transit_boxed(input)
    }
}

impl<'obj, I, O, E, Ef> Clone for DynParser<'obj, I, O, E, Ef>
where
    I: 'obj,
    O: 'obj,
    E: 'obj,
    Ef: 'obj,
{
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}
