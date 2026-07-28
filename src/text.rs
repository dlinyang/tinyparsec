use std::fmt;

use crate::parsec::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct TextPos {
    pub line: u32,
    pub col: u32,
}

impl fmt::Display for TextPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.col + 1)
    }
}

impl TextPos {
    #[inline]
    pub fn walk(&mut self, c: char) {
        if c == '\n' {
            self.col = 0;
            self.line += 1;
        }
        else {
            self.col += 1;
        }
    }
}

#[derive(Debug)]
pub struct Token<T> {
    pub inner: T,
    pub text_pos: TextPos,
}

impl<T> Token<T> {
    #[inline]
    pub fn new(inner: T, text_pos: TextPos) -> Self {
        Self { inner, text_pos }
    }

    pub fn covert<S>(&self, f: impl Fn(&T) -> S) -> Token<S> {
        Token::<S>::new(f(&self.inner), self.text_pos)
    }

    pub fn zip<S>(self, other: Token<S>) -> Token<(T, S)> {
        Token { inner: (self.inner, other.inner), text_pos: self.text_pos }
    }
}

impl<T: fmt::Display> fmt::Display for Token<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " {}, token: {}", self.text_pos, self.inner)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Text<'a> {
    pub inner: &'a str,
    pub text_pos: TextPos,
}

impl<'a> Text<'a> {
    #[inline]
    pub fn new(inner: &'a str, text_pos: TextPos) -> Self {
        Self { inner, text_pos}
    }

    #[inline]
    pub fn starts_with(&self, s: &'a str) -> bool {
        self.inner.starts_with(s)
    }

    #[inline]
    pub fn split_as_token(&self, len: usize) -> (Token<&'a str> , Text<'a>){
        let (start, end) = self.inner.split_at(len);
        let mut text_pos = self.text_pos;
        text_pos.col += len as u32;
        // assumption: no newline in prefix
        // for c in start.chars() {
        //     self.text_pos.walk(c);
        // }
        (Token::new(start, self.text_pos), Text::new(end, text_pos))
    }
}

impl<'a> Iterator for Text<'a> {
    type Item = char;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.inner.chars();
        if let Some(c) = chars.next()  {
            self.inner = chars.as_str();
            self.text_pos.walk(c);
            Some(c)
        }
        else {
            None
        }
    }
}

/// symbolic precedence process
#[derive(Clone, Debug)]
pub struct Op {
    pub symbol: String,
    pub precedence: u32,
    pub associate: Assoc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Assoc {
    Left,
    Right,
}

impl Op {
    pub fn new(symbol: impl Into<String>, precedence: u32, associate: Assoc) -> Self {
        Self { symbol: symbol.into() , precedence, associate }
    }
}

impl<'a> ParsecT<Text<'a>, Token<&'a str>> for Op {
    #[inline]
    fn parse(&self, input: Text<'a>) -> ParseResult<Text<'a>, Token<&'a str>> {
        let s = self.symbol.as_str();
        if input.starts_with(s) {
            Ok(input.split_as_token(s.len()))
        }
        else {
            Err(Default::default())
        }
    }
}

///
#[inline]
pub fn char_fn_pc<'a>(f: impl Fn(char) -> bool) -> impl ParsecT<Text<'a>, Token<char>> {
    move |input: Text<'a>| {
        let mut i =  input;
        match i.next() {
            Some(ch) if f(ch) => Ok((Token::new(ch, input.text_pos), i)),
            _ => Err(Default::default())
        }
    }
}

#[inline]
pub fn str_fn_pc<'a>(f: impl Fn(&'a str) -> usize) -> impl ParsecT<Text<'a>, Token<&'a str>> {
    move |input: Text<'a>| {
        let x = f(input.inner);
        if x > 0 {
            Ok(input.split_as_token(x))
        } else {
            Err(Default::default())
        }
    }
}

pub fn char_pc<'a>(c: char) -> impl ParsecT<Text<'a>, Token<char>> {
    char_fn_pc(move |x| x == c)
}

pub fn str_pc<'a, 'b>(s: &'b str) -> impl ParsecT<Text<'a>, Token<&'a str>> {
    move |input: Text<'a>| {
        if input.starts_with(s) {
            Ok(input.split_as_token(s.len()))
        } else {
            Err(Default::default())
        }
    }
}

pub fn alphabetic<'a>(input: Text<'a>) -> ParseResult<Text<'a>, Token<char>> {
    char_fn_pc(|x| x.is_alphabetic()).parse(input)
}

pub fn alphanum<'a>(input: Text<'a>) -> ParseResult<Text<'a>, Token<char>> {
    char_fn_pc(|x| x.is_alphanumeric()).parse(input)
}

pub fn num<'a>(input: Text<'a>) -> ParseResult<Text<'a>, Token<char>> {
    char_fn_pc(|x| x.is_numeric()).parse(input)
}

#[inline]
pub fn consume_pc<'a>(f: impl Fn(char) -> bool + Copy) -> impl ParsecT<Text<'a>, ()> {
    move |input: Text<'a>| char_fn_pc(f).parse(input).map(|(_, rest)| ((), rest))
}

pub fn newline<'a>(input: Text<'a>) -> ParseResult<Text<'a>, ()> {
    char_pc('\n').parse(input).map(|(_, rest)| ((), rest))
}

#[inline]
pub fn end<'a>(input: Text<'a>) -> ParseResult<Text<'a>, ()> {
    let mut i = input;
    if i.next() == None {
        Ok(((), i))
    }
    else {
        Err(Default::default())
    }
}
