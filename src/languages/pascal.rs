// Phobos IDE - Pascal/Delphi Support (honoring roots)
use crate::languages::LanguageSupport;
pub struct PascalLang{kw:Vec<&'static str>}
impl PascalLang{pub fn new()->Self{Self{kw:vec!["program","unit","uses","interface","implementation","begin","end","procedure","function","var","const","type","record","class","object","array","of","string","integer","real","boolean","char","if","then","else","case","of","while","do","for","to","downto","repeat","until","try","except","finally","raise","nil","true","false","and","or","not","div","mod","in","is","as","constructor","destructor","property","inherited","override","virtual","abstract","published","public","private","protected"]}}}
fn hl(&self,l:&str)->String{let t=l.trim_start();if t.starts_with("//")||t.starts_with("{")||t.starts_with("(*"){return format!("<span class=\"comment\">{}</span>",l);}
let mut o=String::new();let mut cur=String::new();
for c in l.chars(){if c.is_alphabetic()||c=='_'{cur.push(c);continue;}
if !cur.is_empty(){let lw=cur.to_lowercase();if self.kw.contains(&lw.as_str()){o.push_str(&format!("<span class=\"keyword\">{}</span>",cur));}else{o.push_str(&cur);}cur.clear();}o.push(c);}
if !cur.is_empty(){let lw=cur.to_lowercase();if self.kw.contains(&lw.as_str()){o.push_str(&format!("<span class=\"keyword\">{}</span>",cur));}else{o.push_str(&cur);}}o}}
impl LanguageSupport for PascalLang{fn name(&self)->&str{"Pascal"}fn extensions(&self)->Vec<&str>{vec!["pas","pp","dpr","lpr"]}fn syntax_highlight(&self,c:&str)->String{c.lines().map(|l|self.hl(l)).collect::<Vec<_>>().join("\n")+"\n"}}
