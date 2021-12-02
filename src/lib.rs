//! # Examples
//!
//! ```
//! use honeycomb::parser::Parser;
//! use honeycomb::stream::parse_iter_complete;
//! use honeycomb::error::GeneralError;
//! use honeycomb::element::Equals;
//! use honeycomb::character::{AsChar, Digit};
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! pub struct Rgb {
//!     red: u8,
//!     green: u8,
//!     blue: u8,
//! }
//!
//! fn channel_parser<I>() -> impl Parser<I, Output = u8, Error = GeneralError<I>
//! where
//!     I: AsChar
//! {
//!     Digit { base: 16 }.then(Digit { base: 16 }).map(|(high, low)| (high << 4) | low)
//! }
//!
//! pub fn color_parser<I>() -> impl Parser<I, Output = Rgb, Error = GeneralError>
//! where
//!     I: AsChar,
//! {
//!     Equals('#')
//!         .then(channel_parser())
//!         .then(channel_parser())
//!         .then(channel_parser())
//!         .map(|(((_, red), green), blue)| Rgb { red, green, blue })
//! }
//!
//! pub fn parse_color(input: &str) -> Result<Rgb, GeneralError> {
//!     parse_iter_complete(color_parser(), input.chars())
//! }
//! ```

pub mod parser;
pub mod error;
pub mod element;
pub mod character;
pub mod stream;
