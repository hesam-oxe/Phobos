//! Phobos IDE - Emmet Abbreviation Engine
//! Expands abbreviations like `div#main.wrapper>ul.menu>li.item*4{Item $}` into markup.
//!
//! Supported subset:
//! - Elements with `#id`, `.class`, `[attr=value]`, `{text}`
//! - Child `>` and sibling `+` combinators
//! - Multiplication `*N` with `$` numbering (and `$$` zero-padded)
//! - Grouping `(...)` (optionally multiplied)
//! - Implicit tags: `.box` → `div.box`, `ul>.item` → `ul>li.item`
//! - `!` → full HTML5 boilerplate
//! - CSS shortcuts: `m10` → `margin: 10px;`, `db` → `display: block;`, ...
//!
//! Self-contained (no external deps), with unit tests.

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr",
];

const IMPLICIT_TAGS: &[(&str, &str)] = &[
    // parent tag -> default child tag
    ("ul", "li"), ("ol", "li"), ("dl", "dt"), ("table", "tr"),
    ("tbody", "tr"), ("thead", "tr"), ("tfoot", "tr"), ("tr", "td"),
    ("select", "option"), ("optgroup", "option"), ("map", "area"),
];

pub const HTML5_BOILERPLATE: &str = "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n    <meta charset=\"UTF-8\">\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n    <title>Document</title>\n</head>\n<body>\n    \n</body>\n</html>";

/// Languages in which Emmet expansion is active.
pub fn is_supported(language: &str) -> bool {
    matches!(
        language,
        "Web" | "JavaScript" | "TypeScript" | "PHP" | "HTML" | "CSS"
    )
}

/// Is `language` the CSS flavor (different stop-chars and expansion rules)?
pub fn is_css(language: &str) -> bool {
    language == "CSS"
}

/// Characters that terminate a scan backwards from the cursor.
fn stop_chars(css: bool) -> &'static str {
    if css {
        " \t\r\n{};(),"
    } else {
        " \t\r\n<();," // '>' allowed: `ul>li`; '<' terminates (start of real markup)
    }
}

/// Returns (abbreviation, byte_start) of the emmet-like word directly before `cursor`.
pub fn word_before_cursor(text: &str, cursor: usize, css: bool) -> (String, usize) {
    let cursor = cursor.min(text.len());
    if !text.is_char_boundary(cursor) {
        return (String::new(), cursor);
    }
    let stop = stop_chars(css);
    let mut start = cursor;
    for (i, ch) in text.char_indices().rev() {
        if i >= cursor {
            continue;
        }
        if stop.contains(ch) {
            start = i + ch.len_utf8();
            break;
        }
        start = i;
    }
    (text[start..cursor].to_string(), start)
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq)]
struct Node {
    tag: String,
    id: Option<String>,
    classes: Vec<String>,
    attrs: Vec<(String, String)>,
    text: Option<String>,
    repeat: usize,
    children: Vec<Node>,
}

impl Node {
    /// Resolve `$` numbering placeholders with a 1-based counter.
    fn apply_numbering(&mut self, index: usize) {
        let pad = |s: &str| {
            if !s.contains('$') {
                return s.to_string();
            }
            let w = s.matches('$').count();
            if w <= 1 {
                s.replace('$', &index.to_string())
            } else {
                s.replace(
                    &"$".repeat(w),
                    &format!("{:0width$}", index, width = w),
                )
            }
        };
        if let Some(id) = &self.id {
            self.id = Some(pad(id));
        }
        for c in &mut self.classes {
            *c = pad(c);
        }
        for (k, v) in &mut self.attrs {
            *k = pad(k);
            *v = pad(v);
        }
        if let Some(t) = &self.text {
            self.text = Some(pad(t));
        }
    }

    fn is_empty_unit(&self) -> bool {
        self.tag.is_empty()
            && self.id.is_none()
            && self.classes.is_empty()
            && self.attrs.is_empty()
            && self.text.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op {
    Root,
    Child,   // attaches as child of previous node
    Sibling, // attaches with same parent as previous node
}

type Item = (Op, Node);

/// Parse an abbreviation into a flat attach-list. All chars must be consumed.
fn parse(abbr: &str) -> Option<Vec<Item>> {
    let chars: Vec<char> = abbr.chars().collect();
    let mut i = 0usize;
    let items = parse_items(&chars, &mut i)?;
    if i < chars.len() {
        return None;
    }
    Some(items)
}

fn parse_items(c: &[char], i: &mut usize) -> Option<Vec<Item>> {
    let mut items: Vec<Item> = Vec::new();
    let mut op = Op::Root;
    let mut pending_op = false; // trailing '+' or '>' with nothing after
    while *i < c.len() {
        match c[*i] {
            '+' => {
                op = Op::Sibling;
                pending_op = true;
                *i += 1;
            }
            '>' => {
                op = Op::Child;
                pending_op = true;
                *i += 1;
            }
            '(' => {
                // group: find matching ')'
                let mut depth = 1usize;
                let start = *i + 1;
                let mut end = start;
                while end < c.len() {
                    match c[end] {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    end += 1;
                }
                if depth != 0 {
                    return None; // unclosed group
                }
                let inner: Vec<char> = c[start..end].iter().copied().collect();
                let mut j = 0usize;
                let group = parse_items(&inner, &mut j)?;
                if j < inner.len() {
                    return None;
                }
                *i = end + 1;
                // optional multiplier on group
                let mut repeat = 1usize;
                if *i < c.len() && c[*i] == '*' {
                    let (n, ni) = parse_number(c, *i + 1)?;
                    repeat = n;
                    *i = ni;
                }
                // emit group items; extra repeats become siblings
                for r in 0..repeat {
                    for (k, (gop, gnode)) in group.iter().enumerate() {
                        let mut n = gnode.clone();
                        if repeat > 1 {
                            n.apply_numbering(r + 1);
                        }
                        let use_op = if r > 0 && k == 0 { Op::Sibling } else { *gop };
                        items.push((use_op, n));
                    }
                }
                op = Op::Sibling;
                pending_op = false;
            }
            _ => {
                let node = parse_node(c, i)?;
                if node.is_empty_unit() {
                    return None;
                }
                items.push((op, node));
                op = Op::Sibling;
                pending_op = false;
            }
        }
    }
    if items.is_empty() || pending_op {
        return None;
    }
    Some(items)
}

fn parse_number(c: &[char], mut i: usize) -> Option<(usize, usize)> {
    let start = i;
    while i < c.len() && c[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return None;
    }
    let s: String = c[start..i].iter().collect();
    let n: usize = s.parse().ok()?;
    if n == 0 || n > 10_000 {
        return None;
    }
    Some((n, i))
}

fn parse_node(c: &[char], i: &mut usize) -> Option<Node> {
    let mut node = Node::default();
    // tag
    while *i < c.len()
        && (c[*i].is_alphanumeric() || matches!(c[*i], '-' | '_' | ':' | '$'))
    {
        node.tag.push(c[*i]);
        *i += 1;
    }
    loop {
        match c.get(*i) {
            Some('#') => {
                *i += 1;
                let v = parse_ident(c, i);
                if v.is_empty() {
                    return None;
                }
                node.id = Some(v);
            }
            Some('.') => {
                *i += 1;
                let v = parse_ident(c, i);
                if v.is_empty() {
                    return None;
                }
                node.classes.push(v);
            }
            Some('[') => {
                *i += 1;
                let mut attrs_str = String::new();
                let mut depth = 1usize;
                while *i < c.len() {
                    if c[*i] == ']' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    if c[*i] == '[' {
                        depth += 1;
                    }
                    attrs_str.push(c[*i]);
                    *i += 1;
                }
                if *i >= c.len() {
                    return None; // unclosed
                }
                *i += 1; // consume ']'
                for part in attrs_str.split_whitespace() {
                    let (k, v) = match part.split_once('=') {
                        Some((k, v)) => (
                            k.to_string(),
                            v.trim_matches('"').trim_matches('\'').to_string(),
                        ),
                        None => (part.to_string(), part.to_string()),
                    };
                    if k.is_empty() {
                        return None;
                    }
                    node.attrs.push((k, v));
                }
            }
            Some('*') => {
                let (n, ni) = parse_number(c, *i + 1)?;
                node.repeat = n;
                *i = ni;
            }
            Some('{') => {
                *i += 1;
                let mut text = String::new();
                while *i < c.len() && c[*i] != '}' {
                    text.push(c[*i]);
                    *i += 1;
                }
                if *i >= c.len() {
                    return None; // unclosed
                }
                *i += 1;
                node.text = Some(text);
            }
            _ => break,
        }
    }
    Some(node)
}

fn parse_ident(c: &[char], i: &mut usize) -> String {
    let mut s = String::new();
    while *i < c.len()
        && (c[*i].is_alphanumeric() || matches!(c[*i], '-' | '_' | '$'))
    {
        s.push(c[*i]);
        *i += 1;
    }
    s
}

// ---------------------------------------------------------------------------
// Tree building & rendering
// ---------------------------------------------------------------------------

fn build_tree(items: Vec<Item>) -> Vec<Node> {
    // arena with parent links
    let mut arena: Vec<Node> = Vec::new();
    let mut parent: Vec<usize> = Vec::new(); // usize::MAX = root

    for (op, node) in items {
        let p = match op {
            Op::Root => usize::MAX,
            Op::Child => arena.len().saturating_sub(1),
            Op::Sibling => parent.last().copied().unwrap_or(usize::MAX),
        };
        let repeat = node.repeat.max(1);
        for r in 1..=repeat {
            let mut n = node.clone();
            if repeat > 1 {
                n.apply_numbering(r);
            }
            arena.push(n);
            parent.push(p);
        }
    }

    // children index lists
    let n = arena.len();
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut roots: Vec<usize> = Vec::new();
    for i in 0..n {
        match parent[i] {
            usize::MAX => roots.push(i),
            p => children[p].push(i),
        }
    }

    // resolve implicit tags (needs parent tag, so after parenting is known)
    for i in 0..n {
        if arena[i].tag.is_empty() {
            let default_tag = if parent[i] != usize::MAX {
                let pt = arena[parent[i]].tag.clone();
                IMPLICIT_TAGS
                    .iter()
                    .find(|(k, _)| *k == pt)
                    .map(|(_, v)| v.to_string())
            } else {
                None
            };
            arena[i].tag = default_tag.unwrap_or_else(|| "div".to_string());
        }
    }

    // flatten preserving document order
    let mut out: Vec<Node> = Vec::with_capacity(n);
    for r in &roots {
        flatten(&arena, &children, *r, &mut out);
    }
    out
}

/// Recursively flatten an arena node (and its subtree) into document order.
fn flatten(arena: &[Node], children: &[Vec<usize>], idx: usize, out: &mut Vec<Node>) {
    let mut node = arena[idx].clone();
    node.repeat = 1;
    node.children = children[idx]
        .iter()
        .map(|c| {
            let mut sub = Vec::new();
            flatten(arena, children, *c, &mut sub);
            sub.remove(0)
        })
        .collect();
    out.push(node);
}

fn escape_html(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            _ => o.push(ch),
        }
    }
    o
}

fn attrs_str(node: &Node) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(id) = &node.id {
        parts.push(format!("id=\"{}\"", escape_html(id)));
    }
    if !node.classes.is_empty() {
        parts.push(format!("class=\"{}\"", escape_html(&node.classes.join(" "))));
    }
    for (k, v) in &node.attrs {
        parts.push(format!("{}=\"{}\"", escape_html(k), escape_html(v)));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" {}", parts.join(" "))
    }
}

fn render_tree(nodes: &[Node], depth: usize) -> String {
    let mut out = String::new();
    let indent = "    ".repeat(depth);
    for node in nodes {
        let tag = if node.tag.is_empty() { "div" } else { &node.tag };
        let open = format!("<{}{}>", tag, attrs_str(node));
        if VOID_TAGS.contains(&tag) {
            out.push_str(&indent);
            out.push_str(&format!("<{}{}/>\n", tag, attrs_str(node)));
            continue;
        }
        let kids = &node.children;
        match (kids.is_empty(), &node.text) {
            (true, None) => {
                out.push_str(&indent);
                out.push_str(&open);
                out.push_str(&format!("</{}>\n", tag));
            }
            (true, Some(t)) => {
                out.push_str(&indent);
                out.push_str(&open);
                out.push_str(&escape_html(t));
                out.push_str(&format!("</{}>\n", tag));
            }
            (false, text) => {
                out.push_str(&indent);
                out.push_str(&open);
                out.push('\n');
                if let Some(t) = text {
                    out.push_str(&indent);
                    out.push_str("    ");
                    out.push_str(&escape_html(t));
                    out.push('\n');
                }
                out.push_str(&render_tree(kids, depth + 1));
                out.push_str(&indent);
                out.push_str(&format!("</{}>\n", tag));
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// CSS expansion
// ---------------------------------------------------------------------------

/// (shortcut, property) — entries containing ':' are complete (no value needed).
/// Value-required entries take the remainder of the abbreviation as the value.
const CSS_SHORTCUTS: &[(&str, &str)] = &[
    ("db", "display: block"),
    ("df", "display: flex"),
    ("dif", "display: inline-flex"),
    ("dib", "display: inline-block"),
    ("di", "display: inline"),
    ("dn", "display: none"),
    ("dg", "display: grid"),
    ("fl", "float: left"),
    ("fr", "float: right"),
    ("cb", "clear: both"),
    ("psa", "position: absolute"),
    ("psr", "position: relative"),
    ("psf", "position: fixed"),
    ("pss", "position: sticky"),
    ("bxz", "box-sizing: border-box"),
    ("ova", "overflow: auto"),
    ("ovh", "overflow: hidden"),
    ("cur", "cursor"),
    ("curp", "cursor: pointer"),
    ("m", "margin"),
    ("mt", "margin-top"),
    ("mb", "margin-bottom"),
    ("ml", "margin-left"),
    ("mr", "margin-right"),
    ("p", "padding"),
    ("pt", "padding-top"),
    ("pb", "padding-bottom"),
    ("pl", "padding-left"),
    ("pr", "padding-right"),
    ("w", "width"),
    ("h", "height"),
    ("maw", "max-width"),
    ("mah", "max-height"),
    ("miw", "min-width"),
    ("mih", "min-height"),
    ("t", "top"),
    ("b", "bottom"),
    ("l", "left"),
    ("r", "right"),
    ("z", "z-index"),
    ("op", "opacity"),
    ("g", "gap"),
    ("bg", "background"),
    ("bgc", "background-color"),
    ("c", "color"),
    ("fs", "font-size"),
    ("fw", "font-weight"),
    ("ff", "font-family"),
    ("ta", "text-align"),
    ("td", "text-decoration"),
    ("tt", "text-transform"),
    ("lh", "line-height"),
    ("ls", "letter-spacing"),
    ("br", "border-radius"),
    ("bd", "border"),
    ("bdb", "border-bottom"),
    ("bdt", "border-top"),
    ("bdr", "border-right"),
    ("bdl", "border-left"),
    ("tr", "transition"),
    ("trf", "transform"),
    ("ai", "align-items"),
    ("jc", "justify-content"),
    ("fd", "flex-direction"),
];

/// Properties whose numeric value stays unitless.
const CSS_UNITLESS: &[&str] = &["z-index", "font-weight", "opacity", "line-height"];

fn css_value(rest: &str) -> Option<String> {
    if rest.is_empty() {
        return None;
    }
    if rest.starts_with('#') {
        return Some(rest.to_string());
    }
    let first = rest.chars().next()?;
    if first.is_ascii_digit() {
        let split = rest.find(|c: char| !c.is_ascii_digit() && c != '.');
        match split {
            None => Some(format!("{}px", rest)),
            Some(sp) => {
                let (num, unit) = rest.split_at(sp);
                if num.is_empty() {
                    return None;
                }
                let unit = if unit == "p" { "%" } else { unit };
                if !unit.chars().all(|c| c.is_ascii_alphanumeric() || c == '%') {
                    return None;
                }
                Some(format!("{}{}", num, unit))
            }
        }
    } else if rest.chars().all(|c| c.is_ascii_alphabetic()) {
        // single-letter value shortcuts (emmet style): ta:c → text-align: center
        let mapped = match rest {
            "c" => "center",
            "l" => "left",
            "r" => "right",
            "t" => "top",
            "b" => "bottom",
            _ => rest,
        };
        Some(mapped.to_string())
    } else {
        None
    }
}

/// Expand a CSS abbreviation like `m10` → `margin: 10px;`
pub fn expand_css(abbr: &str) -> Option<String> {
    let abbr = abbr.trim();
    if abbr.is_empty()
        || !abbr
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '#' | '-' | '.' | '_'))
    {
        return None;
    }
    // exact shortcuts (complete declarations or value-required failure)
    for (p, full) in CSS_SHORTCUTS {
        if abbr == *p {
            return if full.contains(':') {
                Some(format!("{};", full))
            } else {
                None
            };
        }
    }
    // longest value-required prefix match
    let mut best: Option<(usize, &str)> = None;
    for (p, prop) in CSS_SHORTCUTS {
        if abbr.starts_with(p) && !prop.contains(':') {
            if best.as_ref().map(|(l, _)| p.len() > *l).unwrap_or(true) {
                best = Some((p.len(), prop));
            }
        }
    }
    let (skip, prop) = best?;
    let rest = &abbr[skip..];
    let value = css_value(rest)?;
    let value = if CSS_UNITLESS.contains(&prop) {
        rest.to_string()
    } else {
        value
    };
    Some(format!("{}: {};", prop, value))
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Expand an HTML/JSX-style abbreviation. Returns pretty-printed markup.
pub fn expand(abbr: &str) -> Option<String> {
    let abbr = abbr.trim();
    if abbr.is_empty() {
        return None;
    }
    if abbr == "!" {
        return Some(HTML5_BOILERPLATE.to_string());
    }
    // reject things that look like real markup, not abbreviations
    if abbr.starts_with('<') || abbr.contains("</") {
        return None;
    }
    let items = parse(abbr)?;
    let roots = build_tree(items);
    if roots.is_empty() {
        return None;
    }
    let mut out = render_tree(&roots, 0);
    while out.ends_with('\n') {
        out.pop();
    }
    Some(out)
}

/// Dispatch expansion by language ("CSS" → css rules, else html rules).
pub fn expand_for(lang: &str, abbr: &str) -> Option<String> {
    if is_css(lang) {
        expand_css(abbr)
    } else {
        expand(abbr)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_tag() {
        assert_eq!(expand("p").unwrap(), "<p></p>");
    }

    #[test]
    fn id_class_attrs_text() {
        assert_eq!(
            expand("div#page.section[data-x=1]{hi}").unwrap(),
            "<div id=\"page\" class=\"section\" data-x=\"1\">hi</div>"
        );
    }

    #[test]
    fn child_and_sibling() {
        assert_eq!(
            expand("div>p+span").unwrap(),
            "<div>\n    <p></p>\n    <span></span>\n</div>"
        );
        assert_eq!(
            expand("div>p>span").unwrap(),
            "<div>\n    <p>\n        <span></span>\n    </p>\n</div>"
        );
    }

    #[test]
    fn multiplication_numbering() {
        assert_eq!(
            expand("ul>li.item${Item $}*2").unwrap(),
            "<ul>\n    <li class=\"item1\">Item 1</li>\n    <li class=\"item2\">Item 2</li>\n</ul>"
        );
    }

    #[test]
    fn zero_padded_numbering() {
        assert_eq!(
            expand("img#p$$*3").unwrap(),
            "<img id=\"p01\"/>\n<img id=\"p02\"/>\n<img id=\"p03\"/>"
        );
    }

    #[test]
    fn grouping() {
        assert_eq!(
            expand("(header+main+footer)*2").unwrap(),
            "<header></header>\n<main></main>\n<footer></footer>\n<header></header>\n<main></main>\n<footer></footer>"
        );
    }

    #[test]
    fn implicit_tags() {
        assert_eq!(
            expand("ul>.item").unwrap(),
            "<ul>\n    <li class=\"item\"></li>\n</ul>"
        );
    }

    #[test]
    fn void_tags() {
        assert_eq!(expand("br").unwrap(), "<br/>");
    }

    #[test]
    fn boilerplate() {
        assert!(expand("!").unwrap().starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn rejects_markup_and_garbage() {
        assert!(expand("<div>").is_none());
        assert!(expand("div(").is_none());
        assert!(expand("div>").is_none());
        assert!(expand("div+").is_none());
        assert!(expand("").is_none());
        assert!(expand("*3").is_none());
    }

    #[test]
    fn word_before_cursor_stops_at_space() {
        let text = "hello div.foo world";
        let (w, s) = word_before_cursor(text, text.len(), false);
        assert_eq!(w, "world");
        assert_eq!(s, text.len() - 5);
    }

    #[test]
    fn word_before_cursor_allows_child_op() {
        let text = "ul>li";
        let (w, _) = word_before_cursor(text, text.len(), false);
        assert_eq!(w, "ul>li");
    }

    #[test]
    fn css_expand_basic() {
        assert_eq!(expand_css("m10").unwrap(), "margin: 10px;");
        assert_eq!(expand_css("db").unwrap(), "display: block;");
        assert_eq!(expand_css("w100p").unwrap(), "width: 100%;");
        assert_eq!(expand_css("bgc#fff").unwrap(), "background-color: #fff;");
        assert_eq!(expand_css("fs14").unwrap(), "font-size: 14px;");
        assert_eq!(expand_css("fw700").unwrap(), "font-weight: 700;");
        assert_eq!(expand_css("tac").unwrap(), "text-align: center;");
        assert_eq!(expand_css("mb8").unwrap(), "margin-bottom: 8px;");
        assert_eq!(expand_css("br4").unwrap(), "border-radius: 4px;");
        assert_eq!(expand_css("psa").unwrap(), "position: absolute;");
        assert!(expand_css("qqq").is_none());
        assert!(expand_css("cur").is_none()); // value-required with no value
    }

    #[test]
    fn css_word_stops_at_brace() {
        let text = "body { m10}";
        let (w, _) = word_before_cursor(text, text.len() - 1, true);
        assert_eq!(w, "m10");
    }

    #[test]
    fn expand_for_dispatches() {
        assert_eq!(expand_for("CSS", "m10").unwrap(), "margin: 10px;");
        assert_eq!(expand_for("Web", "br").unwrap(), "<br/>");
        assert!(expand_for("Rust", "br").is_none() == false); // dispatch doesn't filter language
    }
}
