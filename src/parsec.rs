/// simple parser combinator module
use std::result::Result;

///  ParseError trace error with string
#[derive(Debug, Default)]
pub struct ParseError {
    pub errors: Vec<String>,
}

impl ParseError {
    #[inline]
    pub fn new<T: ToString>(error: T) -> Self {
        Self {
            errors: vec![error.to_string()],
        }
    }
}

pub trait ParseErrorT {
    fn merge(a: Self, b: Self) -> Self;
}

impl ParseErrorT for ParseError {
    fn merge(mut a: Self, mut b: Self) -> Self {
        a.errors.append(&mut b.errors);
        a
    }
}

pub type ParseResult<I, T, E = ParseError> = Result<(T, I), E>;

// #[inline]
// pub fn parse_result_ok<I, T, E>(i: I, t: T) -> ParseResult<I, T, E> {
//     Ok((t, i))
// }

/// parser combinator type
pub trait ParsecT<I, T, E = ParseError> {
    fn parse(&self, input: I) -> ParseResult<I, T, E>;

    #[inline]
    fn or(&self, g: impl ParsecT<I, T, E>) -> impl ParsecT<I, T, E>
    where
        I: Copy,
        E: ParseErrorT,
    {
        move |input| match self.parse(input) {
            Ok(x) => Ok(x),
            Err(err) => match g.parse(input) {
                Ok(x) => Ok(x),
                Err(other_err) => Err(ParseErrorT::merge(err, other_err)),
            },
        }
    }

    fn and<R>(&self, other: impl ParsecT<I, R, E>) -> impl ParsecT<I, (T, R), E> {
        move |input| {
            let (a, rest) = self.parse(input)?;
            let (b, rest) = other.parse(rest)?;
            Ok(((a, b), rest))
        }
    }

    fn then<R>(&self, other: impl ParsecT<I, R, E>) -> impl ParsecT<I, R, E> {
        move |input| {
            let (_, rest1) = self.parse(input)?;
            let (v2, rest2) = other.parse(rest1)?;
            Ok((v2, rest2))
        }
    }

    fn terminated<R>(&self, other: impl ParsecT<I, R, E>) -> impl ParsecT<I, T, E> {
        move |input| {
            let (v1, rest1) = self.parse(input)?;
            let (_, rest2) = other.parse(rest1)?;
            Ok((v1, rest2))
        }
    }

    fn preceded<L>(&self, other: impl ParsecT<I, L, E>) -> impl ParsecT<I, T, E> {
        move |input| {
            let (_, rest1) = other.parse(input)?;
            let (v2, rest2) = self.parse(rest1)?;
            Ok((v2, rest2))
        }
    }

    #[inline]
    fn optional(&self, input: I) -> (Option<T>, I)
    where I: Copy
    {
        if let Ok((t, rest)) = self.parse(input) {
            (Some(t), rest)
        } else {
            (None, input)
        }
    }
}

/// impl parser combinator function
impl<I, T, E, F> ParsecT<I, T, E> for F
where
    F: Fn(I) -> ParseResult<I, T, E>,
{
    #[inline]
    fn parse(&self, input: I) -> ParseResult<I, T, E> {
        (self)(input)
    }
}

#[inline]
pub fn lexeme<I: Copy, T, E>(
    ws: impl ParsecT<I, (), E> + Copy,
    t: impl ParsecT<I, T, E>,
) -> impl ParsecT<I, T, E> {
    move |input| {
        let (_, rest) = many0(ws).parse(input)?;
        let (ret, rest) = t.parse(rest)?;
        Ok((ret, rest))
    }
}

pub fn between<I, T, E>(
    left: impl ParsecT<I, T, E>,
    right: impl ParsecT<I, T, E>,
    inner: impl ParsecT<I, T, E>,
) -> impl ParsecT<I, T, E> {
    move |input| {
        let (_, left_rest) = left.parse(input)?;
        let (inner, inner_rest) = inner.parse(left_rest)?;
        let (_, right_rest) = right.parse(inner_rest)?;
        Ok((inner, right_rest))
    }
}

#[macro_export]
macro_rules! choice {
    ($p1:expr, $p2:expr) => { $p1.or($p2) };
    ($p1:expr, $p2:expr, $($rest:expr),+) => { $p1.or(choice!($p2, $($rest),+)) };
}

#[inline]
pub fn many0<I: Copy, T, E>(f: impl ParsecT<I, T, E>) -> impl ParsecT<I, Vec<T>, E> {
    move |mut input| {
        let mut result = Vec::new();
        while let Ok((val, rest)) = f.parse(input) {
            result.push(val);
            input = rest;
        }
        Ok((result, input))
    }
}

#[inline]
pub fn many<I: Copy, T, E>(f: impl ParsecT<I, T, E>) -> impl ParsecT<I, Vec<T>, E> {
    move |input| match f.parse(input) {
        Ok((val, mut rest)) => {
            let mut result = Vec::new();
            result.push(val);
            while let Ok((val1, rest1)) = f.parse(rest) {
                result.push(val1);
                rest = rest1;
            }
            Ok((result, rest))
        }
        Err(e) => Err(e),
    }
}

#[inline]
pub fn optional<I: Copy, T, E>(f: impl ParsecT<I, T, E>, input: I) -> (Option<T>, I) {
    if let Ok((t, rest)) = f.parse(input) {
        (Some(t), rest)
    } else {
        (None, input)
    }
}

// pub fn try_<'a, T>(f: impl ParsecT<'a, T>, input: &'a str) -> ParseResult<'a, T> {
//     match f.parse(input) {
//         Ok((val, rest)) => Ok((val, rest)),
//         Err(err) => Err(err),
//     }
// }
