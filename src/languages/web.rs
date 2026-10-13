// Phobos IDE - Web (HTML/CSS/TS) Support
use crate::languages::LanguageSupport;
pub struct WebLang{kw:Vec<&'static str>}
impl WebLang{pub fn new()->Self{Self{kw:vec!["html","head","body","div","span","a","p","h1","h2","h3","ul","li","table","form","input","button","script","style","link","meta","color","background","margin","padding","border","display","flex","grid","position","width","height","font","text"]}}}
fn hl(&self,l:&str)->String{let mut o=String::new();let mut i=0;let ch:Vec<char>=l.chars().collect();
while i<ch.len(){if ch[i]=='<'{let mut j=i;while j<ch.len()&&ch[j]!='>'{j+=1;}if j<ch.len(){j+=1;}let tag:String=ch[i..j].iter().collect();o.push_str(&format!("<span class=\"keyword\">{}</span>",tag));i=j;continue;}
if i+4<=ch.len(){let s:String=ch[i..(i+4).min(ch.len())].iter().collect();if s=="<!--"{o.push_str(&format!("<span class=\"comment\">{}</span>",&l[i..]));break;}}
o.push(ch[i]);i+=1;}o}}
impl LanguageSupport for WebLang{fn name(&self)->&str{"Web"}fn extensions(&self)->Vec<&str>{vec!["html","css","tsx","jsx","vue","svelte"]}fn syntax_highlight(&self,c:&str)->String{c.lines().map(|l|self.hl(l)).collect::<Vec<_>>().join("\n")+"\n"}}
