// Phobos IDE - Data Langs (YAML/JSON/TOML) Support
use crate::languages::LanguageSupport;
pub struct DataLang;
impl DataLang{pub fn new()->Self{Self}}
impl LanguageSupport for DataLang{
fn name(&self)->&str{"Data"}
fn extensions(&self)->Vec<&str>{vec!["yaml","yml","json","toml","xml","ini"]}
fn syntax_highlight(&self,code:&str)->String{
code.lines().map(|l|{let t=l.trim_start();
if t.starts_with('#'){format!("<span class=\"comment\">{}</span>",l)}
else if let Some(idx)=l.find(':'){let (k,v)=l.split_at(idx);format!("<span class=\"keyword\">{}</span>{}",k,v)}
else if l.contains('"'){format!("<span class=\"type\">{}</span>",l)}
else{l.to_string()}}).collect::<Vec<_>>().join("\n")+"\n"}}
