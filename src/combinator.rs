use crate::parser::{FatalFailure, Parser, RecovFailure, Transition};
use std::{marker::PhantomData, mem};

pub enum OrState<A, E> {
    ParsingBoth,
    LeftDone(Option<RecovFailure<A, E>>),
    RightDone(Option<RecovFailure<A, E>>),
    AlreadyDone,
}

pub struct Or<P, Q, I>
where
    I: Clone,
    P: Parser<I>,
    Q: Parser<I, Output = P::Output, Errors = E::Errors>,
{
    left: P,
    right: Q,
    state: OrState<P::Output, P::Errors>,
    _marker: PhantomData<I>,
}

impl<P, Q, I> Parser<I> for Or<P, Q, I>
where
    I: Clone,
    P: Parser<I>,
    Q: Parser<I, Output = P::Output, Errors = E::Errors>,
{
    type Output = P::Output;
    type Errors = E::Errors;

    fn transit(&mut self, input: I) -> Transition<Self::Output, Self::Errors> {
        match mem::replace(&mut self.state, OrState::AlreadyDone) {
            OrState::ParsingBoth => match self.left.transit(input.clone()) {
                Transition::AlreadyDone => Transition::AlreadyDone,

                Transition::Parsing => match self.right.transit(input) {
                    Transition::AlreadyDone => Transition::AlreadyDone,
                    Transition::Parsing => {
                        self.state = OrState::ParsingBoth;
                        Transition::Parsing
                    },
                    Transition::Success(output) => Transition::Success(output),
                    Transition::Failure(recov) => {
                        self.state = OrState::RightDone(Some(recov));
                        Transition::Parsing
                    },
                    Transition::Fatal(_) => {
                        self.state = OrState::RightDone(None);
                        Transition::Parsing
                    },
                },

                Transition::Success(output) => Transition::Success(output),

                Transition::Failure(recov) => match self.right.transit(input) {
                    Transition::AlreadyDone => Transition::AlreadyDone,
                    Transition::Parsing => {
                        self.state = OrState::LeftDone(Some(recov));
                        Transition::Parsing
                    },
                    Transition::Success(output) => Transition::Success(output),
                    Transition::Failure(_) | Transition::Fatal(_) => {
                        Transition::Failure(recov)
                    },
                },

                Transition::Fatal(fatal) => match self.right.transit(input) {
                    Transition::AlreadyDone => Transition::AlreadyDone,
                    Transition::Parsing => {
                        self.state = OrState::LeftDone(None);
                        Transition::Parsing
                    },
                    Transition::Success(output) => Transition::Success(output),
                    Transition::Fatal(_) => {
                        Transition::Failure(fatal)
                    },
                    Transition::Failure()
                },
            },

            OrState::AlreadyDone => Transition::AlreadyDone,
        }
    }
}
