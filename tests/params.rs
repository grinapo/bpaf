use bpaf::*;

#[test]
fn get_any_simple() {
    let a = short('a').switch();
    let b = any("REST").help("any help");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(Args::from(&["-a", "-b"])).unwrap().1;
    assert_eq!(r, "-b");

    let r = parser.run_inner(Args::from(&["-b", "-a"])).unwrap().1;
    assert_eq!(r, "-b");

    let r = parser.run_inner(Args::from(&["-b=foo", "-a"])).unwrap().1;
    assert_eq!(r, "-b=foo");
}

#[test]
fn get_any_many() {
    let a = short('a').switch();
    let b = any("REST").help("any help").many();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(Args::from(&["-a", "-b"])).unwrap();
    assert_eq!(r.1, &["-b"]);

    let r = parser.run_inner(Args::from(&["-b", "-a"])).unwrap();
    assert_eq!(r.1, &["-b"]);

    let r = parser.run_inner(Args::from(&["-b", "-a", "-b"])).unwrap();
    assert_eq!(r.1, &["-b", "-b"]);
}

#[test]
fn get_any_many2() {
    let parser = any("REST").os().many().to_options();

    let r = parser.run_inner(Args::from(&["-vvv"])).unwrap();
    assert_eq!(r[0], "-vvv");
}
