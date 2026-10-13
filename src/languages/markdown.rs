// Phobos IDE - Markdown Support
use crate::languages::LanguageSupport;
pub struct MarkdownLang;
impl MarkdownLang{pub fn new()->Self{Self}}
impl LanguageSupport for MarkdownLang{
fn name(&self)->&str{"Markdown"}
fn extensions(&self)->Vec<&str>{vec!["md","markdown"]}
fn syntax_highlight(&self,code:&str)->String{
code.lines().map(|l|{let t=l.trim_start();
if t.starts_with('#'){format!("<span class=\"keyword\">{}</span>",l)}
else if t.starts_with("```"){format!("<span class=\"type\">{}</span>",l)}
else if t.starts_with('-')||t.starts_with('*')||t.starts_with('>'){format!("<span class=\"comment\">{}</span>",l)}
else{l.to_string()}}).collect::<Vec<_>>().join("\n")+"\n"}}
