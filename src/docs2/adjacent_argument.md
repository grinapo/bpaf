<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    package: String,
}

fn package() -> impl Parser<String> {
    long("package")
        .short('p')
        .help("Package to use")
        .argument("SPEC")
        .adjacent()
}

pub fn options() -> OptionParser<Options> {
    construct!(Options { package() }).to_options()
}
```

</details>
<details><summary>Derive example</summary>

```no_run
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long, argument("SPEC"), adjacent)]
    /// Package to use
    package: String,
}
```

</details>
<details><summary>Output</summary>


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>-p</b></tt>=<tt><i>SPEC</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-p</b></tt>, <tt><b>--package</b></tt>=<tt><i>SPEC</i></tt></dt>
<dd>Package to use</dd>
<dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Prints help information</dd>
</dl>
</p>
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: "Source Code Pro", monospace;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>


As with regular [`argument`](NamedArg::argument) its `adjacent` variant is required by default


<div class='bpaf-doc'>
$ app <br>
Expected <tt><b>--package</b></tt>=<tt><i>SPEC</i></tt>, pass <tt><b>--help</b></tt> for usage information
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: "Source Code Pro", monospace;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>


But unlike regular variant `adjacent` requires name and value to be separated by `=` only


<div class='bpaf-doc'>
$ app -p=htb<br>
Options { package: "htb" }
</div>


<div class='bpaf-doc'>
$ app --package=bpaf<br>
Options { package: "bpaf" }
</div>


Separating them by space results in parse failure


<div class='bpaf-doc'>
$ app --package htb<br>
Expected <tt><b>--package</b></tt>=<tt><i>SPEC</i></tt>, got <b>--package</b>. Pass <tt><b>--help</b></tt> for usage information<p></p><tt><b>app</b></tt> STRANGE -&gt; <b>--package</b> <tt><b>htb</b></tt>
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: "Source Code Pro", monospace;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>


<div class='bpaf-doc'>
$ app -p htb<br>
Expected <tt><b>--package</b></tt>=<tt><i>SPEC</i></tt>, got <b>-p</b>. Pass <tt><b>--help</b></tt> for usage information<p></p><tt><b>app</b></tt> STRANGE -&gt; <b>-p</b> <tt><b>htb</b></tt>
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: "Source Code Pro", monospace;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>


<div class='bpaf-doc'>
$ app --package<br>
Expected <tt><b>--package</b></tt>=<tt><i>SPEC</i></tt>, got <b>--package</b>. Pass <tt><b>--help</b></tt> for usage information<p></p><tt><b>app</b></tt> STRANGE -&gt; <b>--package</b>
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: "Source Code Pro", monospace;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>

</details>