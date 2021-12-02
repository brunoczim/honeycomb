use crate::{
    error::{ExpectedEndOfInput, UnexpectedEndOfInput},
    parser::{Done, Parser, Parsing},
};

pub fn parse_iter<It, P>(
    mut parser: P,
    iterable: It,
) -> Result<P::Output, P::Error>
where
    It: IntoIterator,
    P: Parser<It::Item>,
    P::Error: From<ExpectedEndOfInput<P::Output>> + From<UnexpectedEndOfInput>,
{
    for element in iterable {
        match parser.transit(element)? {
            Done(item) => return Ok(item),
            Parsing(new_parser) => parser = new_parser,
        }
    }

    Err(P::Error::from(UnexpectedEndOfInput))
}

pub fn parse_iter_complete<It, P>(
    mut parser: P,
    iterable: It,
) -> Result<P::Output, P::Error>
where
    It: IntoIterator,
    P: Parser<Option<It::Item>>,
    P::Error: From<ExpectedEndOfInput<P::Output>> + From<UnexpectedEndOfInput>,
{
    let mut iterator = iterable.into_iter();
    while let Some(element) = iterator.next() {
        match parser.transit(Some(element))? {
            Done(item) => {
                return if iterator.next().is_none() {
                    Ok(item)
                } else {
                    Err(P::Error::from(ExpectedEndOfInput { found: item }))
                }
            },
            Parsing(new_parser) => parser = new_parser,
        }
    }

    match parser.transit(None)? {
        Done(item) => Ok(item),
        Parsing(_) => Err(P::Error::from(UnexpectedEndOfInput)),
    }
}

pub fn parse_iter_incomplete<It, P>(
    mut parser: P,
    iterable: It,
) -> Result<(It::IntoIter, P::Output), P::Error>
where
    It: IntoIterator,
    P: Parser<Option<It::Item>>,
    P::Error: From<UnexpectedEndOfInput>,
{
    let mut iterator = iterable.into_iter();
    while let Some(element) = iterator.next() {
        match parser.transit(Some(element))? {
            Done(item) => return Ok((iterator, item)),
            Parsing(new_parser) => parser = new_parser,
        }
    }

    match parser.transit(None)? {
        Done(item) => Ok((iterator, item)),
        Parsing(_) => Err(P::Error::from(UnexpectedEndOfInput)),
    }
}
