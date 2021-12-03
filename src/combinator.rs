use crate::parser::{Parser, TransitResult, Transition};
use std::{fmt, marker::PhantomData};

pub struct Map<P, I, F, T>
where
    P: Parser<I>,
    F: FnOnce(P::Output) -> T,
{
    inner: P,
    mapper: F,
    _marker: PhantomData<(I, T)>,
}

impl<P, I, F, T> Map<P, I, F, T>
where
    P: Parser<I>,
    F: FnOnce(P::Output) -> T,
{
    pub(super) fn new(inner: P, mapper: F) -> Self {
        Self { inner, mapper, _marker: PhantomData }
    }
}

impl<P, I, F, T> Parser<I> for Map<P, I, F, T>
where
    P: Parser<I>,
    F: FnOnce(P::Output) -> T,
{
    type Output = F::Output;
    type Error = P::Error;
    type Fatal = P::Fatal;

    fn transit(self, input: I) -> TransitResult<Self, I> {
        match self.inner.transit(input)? {
            Transition::Done { output } => {
                Ok(Transition::Done { output: (self.mapper)(output) })
            },
            Transition::Parsing { parser, error } => Ok(Transition::Parsing {
                parser: Map { inner: parser, ..self },
                error,
            }),
        }
    }
}

impl<P, I, F, T> fmt::Debug for Map<P, I, F, T>
where
    P: Parser<I> + fmt::Debug,
    F: FnOnce(P::Output) -> T + fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("Map")
            .field("inner", &self.inner)
            .field("mapper", &self.mapper)
            .finish()
    }
}

impl<P, I, F, T> Clone for Map<P, I, F, T>
where
    P: Parser<I> + Clone,
    F: FnOnce(P::Output) -> T + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            mapper: self.mapper.clone(),
            _marker: PhantomData,
        }
    }
}

impl<P, I, F, T> Copy for Map<P, I, F, T>
where
    P: Parser<I> + Copy,
    F: FnOnce(P::Output) -> T + Copy,
{
}

pub struct MapInput<P, I, J, F>
where
    P: Parser<I>,
    F: FnMut(J) -> I,
{
    inner: P,
    mapper: F,
    _marker: PhantomData<(I, J)>,
}

impl<P, I, J, F> MapInput<P, I, J, F>
where
    P: Parser<I>,
    F: FnMut(J) -> I,
{
    pub(super) fn new(inner: P, mapper: F) -> Self {
        Self { inner, mapper, _marker: PhantomData }
    }
}

impl<P, I, J, F> Parser<J> for MapInput<P, I, J, F>
where
    P: Parser<I>,
    F: FnMut(J) -> I,
{
    type Output = P::Output;
    type Error = P::Error;
    type Fatal = P::Fatal;

    fn transit(mut self, input: J) -> TransitResult<Self, J> {
        match self.inner.transit((self.mapper)(input))? {
            Transition::Done { output } => Ok(Transition::Done { output }),
            Transition::Parsing { parser, error } => Ok(Transition::Parsing {
                parser: MapInput { inner: parser, ..self },
                error,
            }),
        }
    }
}

impl<P, I, J, F> fmt::Debug for MapInput<P, I, J, F>
where
    P: Parser<I> + fmt::Debug,
    F: FnMut(J) -> I + fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("MapInput")
            .field("inner", &self.inner)
            .field("mapper", &self.mapper)
            .finish()
    }
}

impl<P, I, J, F> Clone for MapInput<P, I, J, F>
where
    P: Parser<I> + Clone,
    F: FnMut(J) -> I + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            mapper: self.mapper.clone(),
            _marker: PhantomData,
        }
    }
}

impl<P, I, J, F> Copy for MapInput<P, I, J, F>
where
    P: Parser<I> + Copy,
    F: FnMut(J) -> I + Copy,
{
}

pub struct MapError<P, I, F, E>
where
    P: Parser<I>,
    F: FnMut(P::Error) -> E,
{
    inner: P,
    mapper: F,
    _marker: PhantomData<(I, E)>,
}

impl<P, I, F, E> MapError<P, I, F, E>
where
    P: Parser<I>,
    F: FnMut(P::Error) -> E,
{
    pub(super) fn new(inner: P, mapper: F) -> Self {
        Self { inner, mapper, _marker: PhantomData }
    }
}

impl<P, I, F, E> Parser<I> for MapError<P, I, F, E>
where
    P: Parser<I>,
    F: FnMut(P::Error) -> E,
{
    type Output = P::Output;
    type Error = F::Output;
    type Fatal = P::Fatal;

    fn transit(mut self, input: I) -> TransitResult<Self, I> {
        match self.inner.transit(input)? {
            Transition::Done { output } => Ok(Transition::Done { output }),
            Transition::Parsing { parser, error } => Ok(Transition::Parsing {
                error: error.map(&mut self.mapper),
                parser: Self { inner: parser, ..self },
            }),
        }
    }
}

impl<P, I, F, E> fmt::Debug for MapError<P, I, F, E>
where
    P: Parser<I> + fmt::Debug,
    F: FnMut(P::Error) -> E + fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("MapError")
            .field("inner", &self.inner)
            .field("mapper", &self.mapper)
            .finish()
    }
}

impl<P, I, F, E> Clone for MapError<P, I, F, E>
where
    P: Parser<I> + Clone,
    F: FnMut(P::Error) -> E + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            mapper: self.mapper.clone(),
            _marker: PhantomData,
        }
    }
}

impl<P, I, F, E> Copy for MapError<P, I, F, E>
where
    P: Parser<I> + Copy,
    F: FnMut(P::Error) -> E + Copy,
{
}

pub struct MapFatal<P, I, F, Ef>
where
    P: Parser<I>,
    F: FnOnce(P::Fatal) -> Ef,
{
    inner: P,
    mapper: F,
    _marker: PhantomData<(I, Ef)>,
}

impl<P, I, F, Ef> MapFatal<P, I, F, Ef>
where
    P: Parser<I>,
    F: FnOnce(P::Fatal) -> Ef,
{
    pub(super) fn new(inner: P, mapper: F) -> Self {
        Self { inner, mapper, _marker: PhantomData }
    }
}

impl<P, I, F, Ef> Parser<I> for MapFatal<P, I, F, Ef>
where
    P: Parser<I>,
    F: FnOnce(P::Fatal) -> Ef,
{
    type Output = P::Output;
    type Error = P::Error;
    type Fatal = F::Output;

    fn transit(self, input: I) -> TransitResult<Self, I> {
        match self.inner.transit(input) {
            Ok(Transition::Done { output }) => Ok(Transition::Done { output }),
            Ok(Transition::Parsing { parser, error }) => {
                Ok(Transition::Parsing {
                    parser: Self { inner: parser, ..self },
                    error,
                })
            },
            Err(error) => Err((self.mapper)(error)),
        }
    }
}

impl<P, I, F, Ef> fmt::Debug for MapFatal<P, I, F, Ef>
where
    P: Parser<I> + fmt::Debug,
    F: FnOnce(P::Fatal) -> Ef + fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("MapError")
            .field("inner", &self.inner)
            .field("mapper", &self.mapper)
            .finish()
    }
}

impl<P, I, F, Ef> Clone for MapFatal<P, I, F, Ef>
where
    P: Parser<I> + Clone,
    F: FnOnce(P::Fatal) -> Ef + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            mapper: self.mapper.clone(),
            _marker: PhantomData,
        }
    }
}

impl<P, I, F, Ef> Copy for MapFatal<P, I, F, Ef>
where
    P: Parser<I> + Copy,
    F: FnOnce(P::Fatal) -> Ef + Copy,
{
}

pub struct MapResult<P, I, F, T, Ef>
where
    P: Parser<I>,
    F: FnOnce(Result<P::Output, P::Fatal>) -> Result<T, Ef>,
{
    inner: P,
    mapper: F,
    _marker: PhantomData<(I, T, Ef)>,
}

impl<P, I, F, T, Ef> MapResult<P, I, F, T, Ef>
where
    P: Parser<I>,
    F: FnOnce(Result<P::Output, P::Fatal>) -> Result<T, Ef>,
{
    pub(super) fn new(inner: P, mapper: F) -> Self {
        Self { inner, mapper, _marker: PhantomData }
    }
}

impl<P, I, F, T, Ef> Parser<I> for MapResult<P, I, F, T, Ef>
where
    P: Parser<I>,
    F: FnOnce(Result<P::Output, P::Fatal>) -> Result<T, Ef>,
{
    type Output = T;
    type Error = P::Error;
    type Fatal = Ef;

    fn transit(self, input: I) -> TransitResult<Self, I> {
        let result = match self.inner.transit(input) {
            Ok(Transition::Done { output }) => (self.mapper)(Ok(output)),
            Ok(Transition::Parsing { parser, error }) => {
                return Ok(Transition::Parsing {
                    parser: Self { inner: parser, ..self },
                    error,
                });
            },
            Err(error) => (self.mapper)(Err(error)),
        };

        match result {
            Ok(output) => Ok(Transition::Done { output }),
            Err(error) => Err(error),
        }
    }
}

impl<P, I, F, T, Ef> fmt::Debug for MapResult<P, I, F, T, Ef>
where
    P: Parser<I> + fmt::Debug,
    F: FnOnce(Result<P::Output, P::Fatal>) -> Result<T, Ef> + fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("MapResult")
            .field("inner", &self.inner)
            .field("mapper", &self.mapper)
            .finish()
    }
}

impl<P, I, F, T, Ef> Clone for MapResult<P, I, F, T, Ef>
where
    P: Parser<I> + Clone,
    F: FnOnce(Result<P::Output, P::Fatal>) -> Result<T, Ef> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            mapper: self.mapper.clone(),
            _marker: PhantomData,
        }
    }
}

impl<P, I, F, T, Ef> Copy for MapResult<P, I, F, T, Ef>
where
    P: Parser<I> + Copy,
    F: FnOnce(Result<P::Output, P::Fatal>) -> Result<T, Ef> + Copy,
{
}

pub struct ErrorInto<P, I, E>
where
    P: Parser<I>,
    E: From<P::Error>,
{
    inner: P,
    _marker: PhantomData<(I, E)>,
}

impl<P, I, E> ErrorInto<P, I, E>
where
    P: Parser<I>,
    E: From<P::Error>,
{
    pub(super) fn new(inner: P) -> Self {
        Self { inner, _marker: PhantomData }
    }
}

impl<P, I, E> Parser<I> for ErrorInto<P, I, E>
where
    P: Parser<I>,
    E: From<P::Error>,
{
    type Output = P::Output;
    type Error = E;
    type Fatal = P::Fatal;

    fn transit(self, input: I) -> TransitResult<Self, I> {
        match self.inner.transit(input)? {
            Transition::Done { output } => Ok(Transition::Done { output }),
            Transition::Parsing { parser, error } => Ok(Transition::Parsing {
                parser: Self { inner: parser, ..self },
                error: error.map(Into::into),
            }),
        }
    }
}

impl<P, I, E> fmt::Debug for ErrorInto<P, I, E>
where
    P: Parser<I> + fmt::Debug,
    E: From<P::Error> + fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("ErrorInto").field("inner", &self.inner).finish()
    }
}

impl<P, I, E> Clone for ErrorInto<P, I, E>
where
    P: Parser<I> + Clone,
    E: From<P::Error> + Clone,
{
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), _marker: PhantomData }
    }
}

impl<P, I, E> Copy for ErrorInto<P, I, E>
where
    P: Parser<I> + Copy,
    E: From<P::Error> + Copy,
{
}

pub struct FatalInto<P, I, Ef>
where
    P: Parser<I>,
    Ef: From<P::Fatal>,
{
    inner: P,
    _marker: PhantomData<(I, Ef)>,
}

impl<P, I, Ef> FatalInto<P, I, Ef>
where
    P: Parser<I>,
    Ef: From<P::Fatal>,
{
    pub(super) fn new(inner: P) -> Self {
        Self { inner, _marker: PhantomData }
    }
}

impl<P, I, Ef> Parser<I> for FatalInto<P, I, Ef>
where
    P: Parser<I>,
    Ef: From<P::Fatal>,
{
    type Output = P::Output;
    type Error = P::Error;
    type Fatal = Ef;

    fn transit(self, input: I) -> TransitResult<Self, I> {
        match self.inner.transit(input) {
            Ok(Transition::Done { output }) => Ok(Transition::Done { output }),
            Ok(Transition::Parsing { parser, error }) => {
                Ok(Transition::Parsing {
                    parser: Self { inner: parser, ..self },
                    error,
                })
            },
            Err(error) => Err(error.into()),
        }
    }
}

impl<P, I, Ef> fmt::Debug for FatalInto<P, I, Ef>
where
    P: Parser<I> + fmt::Debug,
    Ef: From<P::Fatal> + fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("FatalInto").field("inner", &self.inner).finish()
    }
}

impl<P, I, Ef> Clone for FatalInto<P, I, Ef>
where
    P: Parser<I> + Clone,
    Ef: From<P::Fatal> + Clone,
{
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), _marker: PhantomData }
    }
}

impl<P, I, Ef> Copy for FatalInto<P, I, Ef>
where
    P: Parser<I> + Copy,
    Ef: From<P::Fatal> + Copy,
{
}
