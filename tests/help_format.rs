use bpaf::*;

#[test]
fn decorations() {
    set_override(false);
    let p = short('p')
        .long("parser")
        .env("BPAF_VARIABLE")
        .argument::<String>("ARG")
        .to_options()
        .descr("descr\ndescr")
        .header("header\nheader")
        .footer("footer\nfooter")
        .version("version")
        .usage("custom {usage}");

    let r = p
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
descr
descr

custom -p ARG

header
header

Available options:
    -p, --parser <ARG>  [env:BPAF_VARIABLE: N/A]
    -h, --help          Prints help information
    -V, --version       Prints version information

footer
footer
";

    assert_eq!(r, expected);
}

#[test]
fn duplicate_items_same_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = short('c').help("c").switch();
    let c2 = short('c').help("c").switch();
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: (-a [-c] | -b [-c])

Available options:
    -a
    -c          c
    -b
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn duplicate_items_dif_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = short('c').help("c1").switch();
    let c2 = short('c').help("c2").switch();
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: (-a [-c] | -b [-c])

Available options:
    -a
    -c          c1
    -b
    -c          c2
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn duplicate_pos_items_same_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = positional::<String>("C").help("C");
    let c2 = positional::<String>("C").help("C");
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: (-a <C> | -b <C>)

Available positional items:
    <C>  C

Available options:
    -a
    -b
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn duplicate_pos_items_diff_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = positional::<String>("C").help("C1");
    let c2 = positional::<String>("C").help("C2");
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: (-a <C> | -b <C>)

Available positional items:
    <C>  C1
    <C>  C2

Available options:
    -a
    -b
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn enum_with_docs() {
    #[derive(Debug, Clone, Bpaf)]
    /// group help
    enum Mode {
        /// help
        ///
        /// absent
        Intel,

        /// help
        ///
        /// Hidden
        Att,
    }

    let r = mode()
        .to_options()
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: (--intel | --att)

Available options:
  group help
        --intel  help
        --att    help

    -h, --help   Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn anywhere_invariant_check() {
    #[derive(Debug, Clone, Bpaf)]
    #[allow(dead_code)]
    #[bpaf(anywhere)]
    struct Foo {
        tag: (),
        #[bpaf(positional)]
        /// help
        name: String,
        #[bpaf(positional)]
        val: String,
    }

    let a = short('a').switch();
    let b = short('b').switch();
    let parser = construct!(a, foo(), b).to_options();

    parser.check_invariants(true);

    let expected = "\
Usage: [-a] --tag ARG ARG [-b]

Available options:
    -a
        --tag <ARG> <ARG>
            <ARG>  help
    -b
    -h, --help     Prints help information
";
    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, expected);
}

#[test]
fn multi_arg_help() {
    let a = long("flag").help("flag help").req_flag(());
    let b = positional::<String>("NAME").help("pos1 help");
    let c = positional::<bool>("STATE").help("pos2 help");
    let combo = construct!(a, b, c).anywhere();
    let verbose = short('v').long("verbose").help("verbose").switch();
    let parser = construct!(verbose, combo).to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: [-v] --flag NAME STATE

Available options:
    -v, --verbose    verbose
        --flag <NAME> <STATE>  flag help
            <NAME>   pos1 help
            <STATE>  pos2 help
    -h, --help       Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn fallback_display_simple_arg() {
    let parser = long("a")
        .help("help for a")
        .argument("NUM")
        .fallback(42)
        .display_fallback()
        .to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "\
Usage: [--a NUM]

Available options:
        --a <NUM>  help for a
                   [default: 42]
    -h, --help     Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn fallback_display_simple_pos() {
    let parser = positional("NUM")
        .help("help for pos")
        .fallback(42)
        .display_fallback()
        .to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: [<NUM>]

Available positional items:
    <NUM>  help for pos
           [default: 42]

Available options:
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn fallback_display_tuple() {
    #[derive(Copy, Clone, Debug)]
    struct Pair(u32, u32);
    impl std::fmt::Display for Pair {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Pair {}, {}", self.0, self.1)
        }
    }

    let a = long("a").help("help for a").argument("NUM");
    let b = long("b").help("help for b").argument("NUM");
    let parser = construct!(a, b)
        .map(|(a, b)| Pair(a, b))
        .fallback(Pair(42, 333))
        .display_fallback()
        .to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: [--a NUM --b NUM]

Available options:
        --a <NUM>  help for a
        --b <NUM>  help for b
                   [default: Pair 42, 333]
    -h, --help     Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn fallback_display_no_help() {
    let parser = long("a")
        .argument("NUM")
        .fallback(42)
        .display_fallback()
        .to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "\
Usage: [--a NUM]

Available options:
        --a <NUM>
                   [default: 42]
    -h, --help     Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn env_fallback_visible() {
    let fonts_dir = long("fonts")
        .env("OIKOS_FONTS")
        .help("Load fonts from this directory")
        .argument::<String>("DIR")
        .optional();

    let system_fonts = long("system-fonts")
        .env("OIKOS_SYSTEM_FONTS")
        .help("Search for additional fonts in system directories")
        .switch();
    let parser = construct!(fonts_dir, system_fonts).to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: [--fonts DIR] [--system-fonts]

Available options:
        --fonts <DIR>   [env:OIKOS_FONTS: N/A]
                        Load fonts from this directory
        --system-fonts  [env:OIKOS_SYSTEM_FONTS: not set]
                        Search for additional fonts in system directories
    -h, --help          Prints help information
";
    assert_eq!(r, expected);
}
