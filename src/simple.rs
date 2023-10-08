use std::{marker::PhantomData, str::FromStr};

use crate::{
    error::Message,
    from_os_str::parse_os_str,
    parsers::{Anything, Argument, Command, Flag, Named, Positional},
    Doc, Error, Meta, Parser, State,
};

/// A building block for your parsers
///
/// This structure implements different methods depending on how it was created - pay attention to
/// the type parameter. Some versions of the structure also implement [`Parser`] trait.
///
/// Currently constructors are
/// - [`short`] or its alias - [`SimpleParser::with_short`]
/// - [`long`] or its alias - [`SimpleParser::with_long`]
/// - [`env`] or its alias - [`SimpleParser::with_env`]
/// - [`positional`] or its alias [`SimpleParser::positional`]
/// - [`any`] or its alias [`SimpleParser::any`]
#[derive(Debug, Clone)]
pub struct SimpleParser<I>(pub(crate) I);

impl SimpleParser<Named> {
    /// An alias for [`short`]
    ///
    /// This method exists only to have all the documentation for simple parsers colleted under the
    /// same structure, you shouldn't use it directly, use [`short`] instead.
    #[deprecated = "You should use `short(s)` instead of `SimpleParser::with_short(s)`"]
    pub fn with_short(name: char) -> Self {
        short(name)
    }

    pub fn short(self, name: char) -> Self {
        Self(self.0.short(name))
    }

    /// An alias for [`long`]
    ///
    /// This method exists only to have all the documentation for simple parsers colleted under the
    /// same structure, you shouldn't use it directly, use [`long`] instead.
    #[deprecated = "You should use `long(l)` instead of `SimpleParser::with_long(l)`"]
    pub fn with_long(name: &'static str) -> Self {
        long(name)
    }

    pub fn long(self, name: &'static str) -> Self {
        Self(self.0.long(name))
    }

    /// An alias for [`env`]
    ///
    /// This method exists only to have all the documentation for simple parsers colleted under the
    /// same structure, you shouldn't use it directly, use [`long`] instead.
    #[deprecated = "You should use `long(l)` instead of `SimpleParser::with_long(l)`"]
    pub fn with_env(name: &'static str) -> Self {
        Self(Named {
            short: Vec::new(),
            long: Vec::new(),
            env: vec![name],
            help: None,
        })
    }

    pub fn env(self, name: &'static str) -> Self {
        Self(self.0.env(name))
    }

    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<Doc>,
    {
        self.0.help = Some(help.into());
        self
    }

    pub fn switch(self) -> SimpleParser<Flag<bool>> {
        SimpleParser(self.0.switch())
    }

    pub fn flag<V>(self, present: V, absent: V) -> SimpleParser<Flag<V>>
    where
        V: Clone + 'static,
    {
        SimpleParser(self.0.flag(present, absent))
    }

    pub fn req_flag<V>(self, present: V) -> SimpleParser<Flag<V>>
    where
        V: Clone + 'static,
    {
        SimpleParser(self.0.req_flag(present))
    }

    pub fn argument<T>(self, metavar: &'static str) -> SimpleParser<Argument<T>>
    where
        T: FromStr + 'static,
    {
        SimpleParser(self.0.argument(metavar))
    }
}

impl<T> SimpleParser<Flag<T>> {
    pub fn help<M>(self, help: M) -> Self
    where
        M: Into<Doc>,
    {
        Self(self.0.help(help))
    }
}

impl<T> SimpleParser<Argument<T>> {
    pub fn adjacent(self) -> Self {
        Self(self.0.adjacent())
    }
}

impl<T: Clone + 'static> Parser<T> for SimpleParser<Flag<T>> {
    fn eval(&self, args: &mut crate::State) -> Result<T, crate::Error> {
        self.0.eval(args)
    }

    fn meta(&self) -> crate::Meta {
        self.0.meta()
    }
}

impl<T> Parser<T> for SimpleParser<Positional<T>>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    fn eval(&self, args: &mut crate::State) -> Result<T, crate::Error> {
        self.0.eval(args)
    }

    fn meta(&self) -> crate::Meta {
        self.0.meta()
    }
}

impl<T> Parser<T> for SimpleParser<Argument<T>>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        let os = self.0.take_argument(args)?;
        match parse_os_str::<T>(os) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(Error(Message::ParseFailed(args.current, err))),
        }
    }

    fn meta(&self) -> Meta {
        if let Some(item) = self.0.item() {
            Meta::from(item)
        } else {
            Meta::Skip
        }
    }
}

/// Parse a [`flag`](NamedArg::flag)/[`switch`](NamedArg::switch)/[`argument`](NamedArg::argument) that has a short name
///
/// You can chain multiple [`short`](NamedArg::short), [`long`](NamedArg::long) and
/// [`env`](NamedArg::env) for multiple names. You can specify multiple names of the same type,
///  `bpaf` would use items past the first one as hidden aliases.
#[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
#[must_use]
pub fn short(name: char) -> SimpleParser<Named> {
    SimpleParser(Named {
        short: vec![name],
        env: Vec::new(),
        long: Vec::new(),
        help: None,
    })
}

/// Parse a [`flag`](NamedArg::flag)/[`switch`](NamedArg::switch)/[`argument`](NamedArg::argument) that has a long name
///
/// You can chain multiple [`short`](NamedArg::short), [`long`](NamedArg::long) and
/// [`env`](NamedArg::env) for multiple names. You can specify multiple names of the same type,
///  `bpaf` would use items past the first one as hidden aliases.
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
#[must_use]
pub fn long(name: &'static str) -> SimpleParser<Named> {
    SimpleParser(Named {
        long: vec![name],
        env: Vec::new(),
        short: Vec::new(),
        help: None,
    })
}

/// Parse an environment variable
///
/// You can chain multiple [`short`](NamedArg::short), [`long`](NamedArg::long) and
/// [`env`](NamedArg::env) for multiple names. You can specify multiple names of the same type,
///  `bpaf` would use items past the first one as hidden aliases.
///
/// For [`flag`](NamedArg::flag) and [`switch`](NamedArg::switch) environment variable being present
/// gives the same result as the flag being present, allowing to implement things like `NO_COLOR`
/// variables:
///
/// ```console
/// $ NO_COLOR=1 app --do-something
/// ```
///
/// If you don't specify a short or a long name - whole argument is going to be absent from the
/// help message. Use it combined with a named or positional argument to have a hidden fallback
/// that wouldn't leak sensitive info.
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
#[must_use]
pub fn env(variable: &'static str) -> SimpleParser<Named> {
    SimpleParser(Named {
        short: Vec::new(),
        long: Vec::new(),
        help: None,
        env: vec![variable],
    })
}

pub fn positional<T>(metavar: &'static str) -> SimpleParser<Positional<T>> {
    SimpleParser(Positional {
        metavar,
        help: None,
        result_type: PhantomData,
        strict: false,
    })
}

impl<T> SimpleParser<Positional<T>> {
    pub fn positional(metavar: &'static str) -> Self {
        SimpleParser(Positional {
            metavar,
            help: None,
            result_type: PhantomData,
            strict: false,
        })
    }

    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<Doc>,
    {
        self.0.help = Some(help.into());
        self
    }

    /// Changes positional parser to be a "strict" positional
    ///
    /// Usually positional items can appear anywhere on a command line:
    /// ```console
    /// $ ls -d bpaf
    /// $ ls bpaf -d
    /// ```
    /// here `ls` takes a positional item `bpaf` and a flag `-d`
    ///
    /// But in some cases it might be useful to have a stricter separation between
    /// positonal items and flags, such as passing arguments to a subprocess:
    /// ```console
    /// $ cargo run --example basic -- --help
    /// ```
    ///
    /// here `cargo` takes a `--help` as a positional item and passes it to the example
    ///
    /// `bpaf` allows to require user to pass `--` for positional items with `strict` annotation.
    /// `bpaf` would display such positional elements differently in usage line as well.
    #[cfg_attr(not(doctest), doc = include_str!("docs2/positional_strict.md"))]
    #[must_use]
    pub fn strict(mut self) -> Self {
        self.0.strict = true;
        self
    }
}

/// Parse a single arbitrary item from a command line
///
/// **`any` is designed to consume items that don't fit into usual
/// flag/switch/argument/positional/command classification, in most cases you don't need to use
/// it**.
///
/// Type parameter `I` is used for intermediate value, normally you'd use [`String`] or
/// [`OsString`]. This parameter only exists to make it possible to work with non-utf8 encoded
/// arguments such as some rare file names, as well as not having to deal with `OsString` if all
/// you want to process is a string that utf8 can correctly represent.
///
/// Type parameter `T` is a type the parser actually produces.
///
/// Parameter `check` takes an intermediate value (`String` or `OsString`) and decides if `any`
/// parser is going to take it by returning `Some` value or `None` if this is not an expected value
/// for this parser.
///
/// By default, `any` behaves similarly to [`positional`] so you should be using it near the
/// rightmost end of the consumer struct, after all the named parsers and it will only try to parse
/// the first unconsumed item on the command line. It is possible to lift this restriction by
/// calling [`anywhere`](SimpleParser::anywhere) on the parser.
///
pub fn any<I, T, F>(metavar: &str, check: F) -> SimpleParser<Anything<T>>
where
    I: FromStr + 'static,
    F: Fn(I) -> Option<T> + 'static,

    <I as std::str::FromStr>::Err: std::fmt::Display,
{
    SimpleParser(Anything {
        metavar: [(metavar, crate::buffer::Style::Metavar)][..].into(),
        help: None,
        check: Box::new(move |os: std::ffi::OsString| {
            match crate::from_os_str::parse_os_str::<I>(os) {
                Ok(v) => check(v),
                Err(_) => None,
            }
        }),
        anywhere: false,
    })
}

impl<T> SimpleParser<Anything<T>> {
    pub fn anywhere(mut self) -> Self {
        self.0.anywhere = true;
        self
    }

    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<Doc>,
    {
        self.0.help = Some(help.into());
        self
    }

    //    /// Replace metavar with a custom value
    //    /// See examples in [`any`]
    //   #[must_use]
    pub fn metavar<M: Into<Doc>>(mut self, metavar: M) -> Self {
        self.0.metavar = metavar.into();
        self
    }
}

impl<T> Parser<T> for SimpleParser<Anything<T>> {
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        self.0.eval(args)
    }

    fn meta(&self) -> Meta {
        self.0.meta()
    }
}

/// A specialized version of [`any`] that consumes an arbitrary string
///
/// By default `literal` behaves similarly to [`positional`] so you should be using it near the
/// rightmost end of the consumer struct and it will only try to parse the first unconsumed
/// item on the command line. It is possible to lift this restriction by calling
/// [`anywhere`](SimpleParser::anywhere) on the parser.
///
/// Apart from matching to a specific literal, this function behaves similarly to:
/// [`req_flag`](SimpleParser::req_flag) it produces a value it was given or fails with "item not
/// found" error which you can handle with [`fallback`](Parser::fallback),
/// [`optional`](Parser::optional) or by combining several `literal` parsers together.
///
#[cfg_attr(not(doctest), doc = include_str!("_docs/literal.md"))]
///
/// # See also
/// - [`any`] - a generic version of `literal` that uses function to decide if value is to be parsed
/// or not.
/// - [`req_flag`](SimpleParser::req_flag) - parse a short/long flag from a command line or fail with "item not found"
#[must_use]
pub fn literal<T>(literal: &'static str, value: T) -> SimpleParser<Anything<T>>
where
    T: Clone + 'static,
{
    SimpleParser(Anything {
        metavar: [(literal, crate::buffer::Style::Literal)][..].into(),
        help: None,
        check: Box::new(move |os| {
            if os == literal {
                Some(value.clone())
            } else {
                None
            }
        }),
        anywhere: false,
    })
}

impl<T> Parser<T> for SimpleParser<Command<T>> {
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        self.0.eval(args)
    }

    fn meta(&self) -> Meta {
        self.0.meta()
    }
}
