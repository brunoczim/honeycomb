#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecovFailure<A, E> {
    pub output: A,
    pub errors: E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FatalFailure<E> {
    pub errors: E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transition<A, E> {
    AlreadyDone,
    Parsing,
    Success(A),
    Failure(RecovFailure<A, E>),
    Fatal(FatalFailure<E>),
}

impl<A, E> Transition<A, E> {
    pub fn map_output<F, B>(self, mapper: F) -> Transition<B, E>
    where
        F: FnOnce(A) -> B,
    {
        match self {
            Transition::AlreadyDone => Transition::AlreadyDone,
            Transition::Parsing => Transition::Parsing,
            Transition::Success(output) => Transition::Success(mapper(output)),
            Transition::Failure(recov) => Transition::Failure(RecovFailure {
                output: mapper(recov.output),
                errors: recov.errors,
            }),
            Transition::Fatal(fatal) => Transition::Fatal(fatal),
        }
    }

    pub fn map_errors<F, E1>(self, mapper: F) -> Transition<A, E1>
    where
        F: FnOnce(E) -> E1,
    {
        match self {
            Transition::AlreadyDone => Transition::AlreadyDone,
            Transition::Parsing => Transition::Parsing,
            Transition::Success(output) => Transition::Success(output),
            Transition::Failure(recov) => Transition::Failure(RecovFailure {
                output: recov.output,
                errors: mapper(recov.errors),
            }),
            Transition::Fatal(fatal) => {
                Transition::Fatal(FatalFailure { errors: mapper(fatal.errors) })
            },
        }
    }
}

pub trait Parser<I> {
    type Output;
    type Errors: IntoIterator;

    fn transit(&mut self, input: I) -> Transition<Self::Output, Self::Errors>;
}

impl<'this, T, I> Parser<I> for &'this mut T
where
    T: Parser<I> + ?Sized,
{
    type Output = T::Output;
    type Errors = T::Errors;

    fn transit(&mut self, input: I) -> Transition<Self::Output, Self::Errors> {
        (**self).transit(input)
    }
}

impl<T, I> Parser<I> for Box<T>
where
    T: Parser<I> + ?Sized,
{
    type Output = T::Output;
    type Errors = T::Errors;

    fn transit(&mut self, input: I) -> Transition<Self::Output, Self::Errors> {
        (**self).transit(input)
    }
}
