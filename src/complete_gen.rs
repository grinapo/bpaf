// completion:
// static: flag names, command names
// dynamic: argument values, positional item values
//
// for static when running collect any parser that fails
//
// OR: combine completions
// AND: usual logic without shortcircuits
//
// for static completion it's enough to collect items
// for argument completion - only one argument(Comp::Meta) should be active at once
//
// for rendering prefer longer version of names
//
// complete short names to long names if possible

use crate::{
    args::Arg,
    complete_run::Style,
    item::{Item, ShortLong},
    Args, Error,
};
use std::ffi::OsStr;

#[derive(Clone, Debug)]
pub(crate) struct Complete {
    pub(crate) style: Style,
    pub(crate) comps: Vec<Comp>,
    pub(crate) touching: bool,
}

impl Complete {
    pub(crate) fn new(touching: bool, style: Style) -> Self {
        Self {
            comps: Vec::new(),
            touching,
            style,
        }
    }

    pub(crate) fn push_item(&mut self, item: Item, depth: usize) {
        self.comps.push(Comp::Item { item, depth })
    }

    pub(crate) fn push_metadata(
        &mut self,
        meta: &'static str,
        help: Option<String>,
        depth: usize,
        arg: bool,
    ) {
        self.comps.push(Comp::Meta {
            meta,
            depth,
            is_arg: arg,
            help,
        })
    }

    pub(crate) fn push_value(&mut self, body: String, help: Option<String>, depth: usize) {
        self.comps.push(Comp::Value { body, help, depth });
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Comp {
    /// comes from named items, part of "static" completion
    Item { item: Item, depth: usize },

    /// comes from completed values, part of "dynamic" completion
    Value {
        body: String,
        help: Option<String>,
        depth: usize,
    },

    /// Placeholder completion - static completion
    Meta {
        meta: &'static str,
        depth: usize,
        /// true for argument metas, at this moment using other completion items isn't valid
        /// false for positional metas - other items are valid
        is_arg: bool,
        help: Option<String>,
    },
}

impl Comp {
    fn depth(&self) -> usize {
        match self {
            Comp::Item { depth, .. } | Comp::Value { depth, .. } | Comp::Meta { depth, .. } => {
                *depth
            }
        }
    }
}

#[derive(Debug)]
struct ShowComp<'a> {
    /// completion description, only rendered if there's several of them
    descr: &'a Option<String>,

    /// substitution to use for multiple items, unlike subst1 includes metavars
    subst: String,

    /// substitution to use for a single item
    subst1: String,

    is_value: bool,
}

impl Arg {
    fn and_os_string(&self) -> Option<(&Self, &OsStr)> {
        match self {
            Arg::Short(_, s) => {
                if s.is_empty() {
                    None
                } else {
                    Some((self, s))
                }
            }
            Arg::Long(_, s) => Some((self, s)),
            Arg::Word(w) => Some((self, &w.os)),
        }
    }
}

fn pair_to_os_string<'a>(pair: (&'a Arg, &'a OsStr)) -> Option<(&'a Arg, &'a str)> {
    Some((pair.0, pair.1.to_str()?))
}

impl Args {
    pub(crate) fn check_complete(&self) -> Result<(), Error> {
        if let Some(comp) = &self.comp {
            let pair = if comp.touching {
                self.items
                    .iter()
                    .rev()
                    .find_map(Arg::and_os_string)
                    .and_then(pair_to_os_string)
            } else {
                None
            };
            if comp.touching && pair.is_none() {
                // can't do much completing with non-utf8 values since bpaf needs to print them to stdout
                return Err(Error::Stdout("\n".to_string()));
            }

            if let Some((Arg::Short(..), s)) = pair {
                if s.chars().count() > 2 {
                    // don't bother trying to expand -vvvv for now:
                    // -vvv<TAB> => -vvv _
                    return Err(Error::Stdout(format!("{}\n", s)));
                }
            }

            Err(Error::Stdout(comp.complete(pair.map(|pair| pair.1))?))
        } else {
            Ok(())
        }
    }
}

fn preferred_name(name: ShortLong) -> String {
    match name {
        ShortLong::Short(s) => format!("-{}", s),
        ShortLong::Long(l) | ShortLong::ShortLong(_, l) => format!("--{}", l),
    }
}

// check if argument can possibly match the argument passed in and returns a preferrable replacement
fn arg_matches(arg: Option<&str>, name: ShortLong) -> Option<String> {
    let arg = match arg {
        Some(arg) => arg,
        None => return Some(preferred_name(name)),
    };

    let mut can_match = arg == "-";

    match name {
        ShortLong::Long(_) => {}
        ShortLong::Short(s) | ShortLong::ShortLong(s, _) => {
            can_match |= arg
                .strip_prefix('-')
                .and_then(|a| a.strip_prefix(s))
                .map_or(false, |s| s.is_empty());
        }
    }

    match name {
        ShortLong::Short(_) => {}
        ShortLong::Long(l) | ShortLong::ShortLong(_, l) => {
            can_match |= arg.strip_prefix("--").map_or(false, |s| l.starts_with(s));
        }
    }

    if can_match {
        Some(preferred_name(name))
    } else {
        None
    }
}
fn cmd_matches(arg: Option<&str>, name: &'static str, short: Option<char>) -> Option<&'static str> {
    match arg {
        Some(cur) => {
            if name.starts_with(cur) || short.map_or(false, |s| cur == s.to_string()) {
                Some(name)
            } else {
                None
            }
        }
        None => Some(name),
    }
}

impl Complete {
    pub fn complete(&self, arg: Option<&str>) -> Result<String, std::fmt::Error> {
        let mut items: Vec<ShowComp> = Vec::new();
        let max_depth = self.comps.iter().map(Comp::depth).max().unwrap_or(0);
        let mut has_values = false;

        for item in self.comps.iter().filter(|c| c.depth() == max_depth) {
            match item {
                Comp::Item { item, depth: _ } => match item {
                    Item::Positional { metavar, help } => todo!("{:?} {:?}", metavar, help),
                    Item::Command {
                        name,
                        short,
                        help,
                        meta: _,
                    } => {
                        if let Some(long) = cmd_matches(arg, name, *short) {
                            items.push(ShowComp {
                                subst: long.to_string(),
                                subst1: long.to_string(),
                                descr: help,
                                is_value: false,
                            });
                        }
                    }
                    Item::Flag { name, help } => {
                        if let Some(long) = arg_matches(arg, *name) {
                            items.push(ShowComp {
                                subst: long.clone(),
                                subst1: long,
                                descr: help,
                                is_value: false,
                            });
                        }
                    }
                    Item::Argument {
                        name,
                        metavar,
                        env: _,
                        help,
                    } => {
                        if let Some(long) = arg_matches(arg, *name) {
                            items.push(ShowComp {
                                subst: format!("{} <{}>", long, metavar),
                                subst1: long,
                                descr: help,
                                is_value: false,
                            });
                        }
                    }
                },
                Comp::Value {
                    body,
                    help,
                    depth: _,
                } => {
                    has_values = true;
                    items.push(ShowComp {
                        descr: help,
                        subst: body.clone(),
                        subst1: body.clone(),
                        is_value: true,
                    })
                }
                Comp::Meta {
                    meta,
                    depth: _,
                    is_arg,
                    help,
                } => {
                    if *is_arg {
                        return Ok(match arg {
                            Some(val) => format!("{}\n", val),
                            None => format!("<{}>\n", meta),
                        });
                    } else {
                        items.push(ShowComp {
                            descr: help,
                            subst: format!("<{}>", meta),
                            subst1: format!("<{}>", meta),
                            is_value: false,
                        })
                    }
                }
            }
        }

        if has_values {
            items.retain(|i| i.is_value);
        }

        use std::fmt::Write;
        let mut res = String::new();
        if items.len() == 1 {
            writeln!(res, "{}", items[0].subst1)?;
        } else {
            let max_width = items
                .iter()
                .map(|s| s.subst.chars().count())
                .max()
                .unwrap_or(0);

            for item in &items {
                match (self.style, item.descr) {
                    (Style::Bash, None) => writeln!(res, "{}", item.subst),
                    (Style::Bash, Some(descr)) => writeln!(
                        res,
                        "{:padding$}  {}",
                        item.subst,
                        descr,
                        padding = max_width
                    ),
                    (Style::Zsh, None) => writeln!(res, "{}", item.subst1),
                    (Style::Zsh, Some(descr)) => writeln!(res, "{}\t{}", item.subst1, descr),
                }?
            }
        }
        Ok(res)
    }
}

// to allow using ? inside check_complete
impl From<std::fmt::Error> for Error {
    fn from(_: std::fmt::Error) -> Self {
        Error::Stderr("Couldn't render completion info".to_string())
    }
}
