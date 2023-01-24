use eyre::Result;
use html_escape;
use lazy_static::lazy_static;
use pulldown_cmark::{CodeBlockKind, Event};
use regex::Regex;
use slugify::slugify;
use syntect::dumps::from_binary;
use syntect::parsing::SyntaxReference;
use syntect::{
    dumps, highlighting::ThemeSet, html::ClassStyle, html::ClassedHTMLGenerator,
    parsing::SyntaxSet, util::LinesWithEndings,
};

lazy_static! {
    static ref SS: SyntaxSet = syntax_set();
}

lazy_static! {
    static ref BLOCK_CODE_SPEC: Regex = Regex::new(r"^\w+$").unwrap();
}

lazy_static! {
    pub static ref THEMES: ThemeSet = from_binary(include_bytes!("../../syntaxes/all.themedump"));
}


// -- highlighting setup --

fn syntax_set() -> SyntaxSet {
    dumps::from_uncompressed_data(include_bytes!("../../syntaxes/syntax_set.packdump")).unwrap()
}

// map language name from code block to what syntect wants.
fn syntect_lang_name(lang: &str) -> &str {
    match lang {
        "elixir" => "Elixir",
        "clojure" => "Clojure",
        "haskell" => "Haskell",
        "perl" => "Perl",
        "python" => "Python",
        "python3" => "Python",
        "racket" => "Racket",
        "ruby" => "Ruby",
        "rust" => "Rust",
        "shell" => "Shell-Unix-Generic",
        "elm" => "Elm",

        x => x,
    }
}

fn parse_code_spec(lang: &str) -> Option<String> {
    if lang.is_empty() {
        return None;
    }
    if BLOCK_CODE_SPEC.is_match(lang) {
        Some(lang.to_string())
    } else {
        panic!("couldn't match code spec: `{}`", lang)
    }
}



// -- highlighting integrated into parser --

pub struct CodeBlockSyntaxHighlight<'a, I: Iterator<Item = Event<'a>>> {
    parent: I,
}

impl<'a, I: Iterator<Item = Event<'a>>> CodeBlockSyntaxHighlight<'a, I> {
    pub fn new(parent: I) -> Self {
        Self { parent }
    }
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for CodeBlockSyntaxHighlight<'a, I> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let lang = match self.parent.next()? {
            Event::Start(pulldown_cmark::Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => lang,
            other => return Some(other),
        };
        let lang = parse_code_spec(&lang);

        let mut code = String::new();
        while let Some(Event::Text(text)) = self.parent.next() {
            code.push_str(&text);
        }

        let mut res = String::new();
        res.push_str(r#"<pre class="syntax-code">"#);
        push_code_highlight(&mut res, lang, &code);
        res.push_str("</pre>");

        Some(Event::Html(res.into()))
    }
}

fn push_code_highlight<S: AsRef<str>>(s: &mut String, lang: Option<S>, code: &str) {
    if let Some(spec) = lang.and_then(|x| HighlightSpec::find(x.as_ref())) {
        match highlight(&spec, code) {
            Ok(highlight) => {
                s.push_str(r#"<code class="highlight code "#);
                s.push_str(&spec.html_id);
                s.push_str(r#"">"#);
                s.push_str(&highlight);
                s.push_str("</code>");
            }

            Err(err) => {
                panic!();
            }
        }
    } else {
        s.push_str("<code>");
        s.push_str(&html_escape::encode_safe(&code));
        s.push_str("</code>");
    }
}

fn highlight(spec: &HighlightSpec, code: &str) -> Result<String> {
    lazy_static! {
        static ref SPAN_WRAPPER: Regex = Regex::new(r"^<span [^>]+>(?s)(.+)</span>$").unwrap();
    }

    let mut html_generator =
        ClassedHTMLGenerator::new_with_class_style(spec.syntax, &SS, ClassStyle::SpacedPrefixed{prefix: "syntax-"});
    for line in LinesWithEndings::from(code) {
        html_generator.parse_html_for_line_which_includes_newline(line)?
    }

    let generated = html_generator.finalize();

    let cap = SPAN_WRAPPER
        .captures(&generated)
        .expect("Failed toi match syntax span");

    Ok(cap[1].trim().to_string())
}

struct HighlightSpec<'a> {
    pub html_id: String,
    pub syntax: &'a SyntaxReference,
}

impl<'a> HighlightSpec<'a> {
    fn find(original: &str) -> Option<Self> {
        if original.is_empty() {
            return None;
        }

        let mapped = syntect_lang_name(original);

        let syntax = SS
            .find_syntax_by_name(mapped)
            .or_else(|| SS.find_syntax_by_extension(mapped))?;

        Some(Self {
            html_id: slugify!(&syntax.name),
            syntax,
        })
    }
}


// Syntect stuff

pub fn dump_syntax_binary() {
    let file = "syntaxes/syntax_set.packdump";
    println!("Dumping syntax binary to {}", file);
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    builder.add_from_folder("syntaxes", true).unwrap();
    let ss = builder.build();
    dumps::dump_to_uncompressed_file(&ss, file).unwrap();
}



#[cfg(test)]
mod tests {
    use pulldown_cmark::{html, Options, Parser};

    use super::CodeBlockSyntaxHighlight;

    fn convert(s: &str) -> String {
        let transformed = Parser::new_ext(s, Options::all());
        let transformed = CodeBlockSyntaxHighlight::new(transformed);
        let mut body = String::new();
        html::push_html(&mut body, transformed);
        body
    }

    #[test]
    fn test_thing() {
        let s = r"
```rust
let x = 2;
```
";
        let res = convert(s);
        println!("{:?}", res);
        assert!(res.starts_with(r#"<pre><code class="language-rust">"#))
    }
}
