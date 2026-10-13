// Phobos IDE - Swift Support
use crate::languages::LanguageSupport;
pub struct SwiftLang{kw:Vec<&'static str>}
impl SwiftLang{pub fn new()->Self{Self{kw:vec!["import","class","struct","enum","protocol","extension","func","var","let","const","if","else","guard","switch","case","default","for","while","repeat","return","break","continue","fallthrough","do","try","catch","throw","throws","rethrows","in","is","as","self","Self","super","nil","true","false","where","associatedtype","typealias","init","deinit","subscript","operator","precedencegroup","infix","prefix","postfix","lazy","weak","unowned","mutating","nonmutating","override","final","open","public","internal","fileprivate","private","static","async","await","some","any"]}}
fn hl(&self,l:&str)->String{let mut o=String::new();let mut ins=false;let mut cur=String::new();let ch:Vec<char>=l.chars().collect();let mut i=0;
while i<ch.len(){let c=ch[i];if !ins&&i+1<ch.len()&&ch[i]=='/'&&ch[i+1]=='/'{o.push_str(&format!("<span class=\"comment\">{}</span>",&l[i..]));break;}
if c=='"'{ins=!ins;o.push(c);i+=1;continue;}if ins{o.push(c);i+=1;continue;}
if c.is_alphabetic()||c=='_'{cur.push(c);i+=1;continue;}
if !cur.is_empty(){let w=cur.as_str();if self.kw.contains(&w){o.push_str(&format!("<span class=\"keyword\">{}</span>",w));}else{o.push_str(w);}cur.clear();}o.push(c);i+=1;}
if !cur.is_empty(){let w=cur.as_str();if self.kw.contains(&w){o.push_str(&format!("<span class=\"keyword\">{}</span>",w));}else{o.push_str(w);}}o}
}
impl LanguageSupport for SwiftLang{fn name(&self)->&str{"Swift"}fn extensions(&self)->Vec<&str>{vec!["swift"]}fn syntax_highlight(&self,c:&str)->String{c.lines().map(|l|self.hl(l)).collect::<Vec<_>>().join("\n")+"\n"}}
