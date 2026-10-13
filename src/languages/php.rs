// Phobos IDE - PHP Support
use crate::languages::LanguageSupport;
pub struct PhpLang{kw:Vec<&'static str>}
impl PhpLang{pub fn new()->Self{Self{kw:vec!["php","echo","print","function","class","interface","trait","extends","implements","public","private","protected","static","const","var","new","return","if","else","elseif","endif","switch","case","default","break","continue","for","foreach","as","while","do","try","catch","finally","throw","namespace","use","require","include","require_once","include_once","global","isset","empty","exit","die","list","array","callable","clone","instanceof","yield","fn","match","null","true","false"]}}
fn hl(&self,l:&str)->String{let mut o=String::new();let mut ins=false;let mut cur=String::new();let ch:Vec<char>=l.chars().collect();let mut i=0;
while i<ch.len(){let c=ch[i];if !ins&&i+1<ch.len()&&ch[i]=='/'&&ch[i+1]=='/'{o.push_str(&format!("<span class=\"comment\">{}</span>",&l[i..]));break;}
if c=='"'||c=='\''{ins=!ins;o.push(c);i+=1;continue;}if ins{o.push(c);i+=1;continue;}
if c.is_alphabetic()||c=='_'||c=='$'{cur.push(c);i+=1;continue;}
if !cur.is_empty(){let w=cur.as_str();if self.kw.contains(&w)||w.starts_with('$'){o.push_str(&format!("<span class=\"keyword\">{}</span>",w));}else{o.push_str(w);}cur.clear();}o.push(c);i+=1;}
if !cur.is_empty(){o.push_str(cur.as_str());}o}}
impl LanguageSupport for PhpLang{fn name(&self)->&str{"PHP"}fn extensions(&self)->Vec<&str>{vec!["php"]}fn syntax_highlight(&self,c:&str)->String{c.lines().map(|l|self.hl(l)).collect::<Vec<_>>().join("\n")+"\n"}}
