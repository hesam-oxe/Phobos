// Phobos IDE - Ruby Support
use crate::languages::LanguageSupport;
pub struct RubyLang{kw:Vec<&'static str>}
impl RubyLang{pub fn new()->Self{Self{kw:vec!["def","class","module","end","if","else","elsif","unless","case","when","then","while","until","for","in","do","return","break","next","redo","retry","yield","self","nil","true","false","and","or","not","alias","defined","BEGIN","END","include","extend","attr_reader","attr_writer","attr_accessor","private","public","protected","require","load","lambda","proc"]}}
fn hl(&self,l:&str)->String{let mut o=String::new();let mut ins=false;let mut cur=String::new();let ch:Vec<char>=l.chars().collect();let mut i=0;
while i<ch.len(){let c=ch[i];if !ins&&c=='#'{o.push_str(&format!("<span class=\"comment\">{}</span>",&l[i..]));break;}
if c=='"'||c=='\''{ins=!ins;o.push(c);i+=1;continue;}if ins{o.push(c);i+=1;continue;}
if c.is_alphabetic()||c=='_'{cur.push(c);i+=1;continue;}
if !cur.is_empty(){let w=cur.as_str();if self.kw.contains(&w){o.push_str(&format!("<span class=\"keyword\">{}</span>",w));}else{o.push_str(w);}cur.clear();}o.push(c);i+=1;}
if !cur.is_empty(){let w=cur.as_str();if self.kw.contains(&w){o.push_str(&format!("<span class=\"keyword\">{}</span>",w));}else{o.push_str(w);}}o}}
impl LanguageSupport for RubyLang{fn name(&self)->&str{"Ruby"}fn extensions(&self)->Vec<&str>{vec!["rb"]}fn syntax_highlight(&self,c:&str)->String{c.lines().map(|l|self.hl(l)).collect::<Vec<_>>().join("\n")+"\n"}}
