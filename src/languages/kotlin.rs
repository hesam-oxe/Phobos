// Phobos IDE - Kotlin Support
use crate::languages::LanguageSupport;
pub struct KotlinLang{kw:Vec<&'static str>}
impl KotlinLang{pub fn new()->Self{Self{kw:vec!["package","import","class","interface","object","fun","val","var","const","if","else","when","for","while","do","return","break","continue","try","catch","finally","throw","in","is","as","this","super","null","true","false","data","sealed","enum","annotation","companion","init","constructor","out","by","get","set","where","typealias","typeof","suspend","inline","noinline","crossinline","reified","tailrec","operator","infix","open","public","private","protected","internal","final","abstract","override","lateinit","vararg","suspend"]}}
fn hl(&self,l:&str)->String{let mut o=String::new();let mut ins=false;let mut cur=String::new();let ch:Vec<char>=l.chars().collect();let mut i=0;
while i<ch.len(){let c=ch[i];if !ins&&i+1<ch.len()&&ch[i]=='/'&&ch[i+1]=='/'{o.push_str(&format!("<span class=\"comment\">{}</span>",&l[i..]));break;}
if c=='"'{ins=!ins;o.push(c);i+=1;continue;}if ins{o.push(c);i+=1;continue;}
if c.is_alphabetic()||c=='_'{cur.push(c);i+=1;continue;}
if !cur.is_empty(){let w=cur.as_str();if self.kw.contains(&w){o.push_str(&format!("<span class=\"keyword\">{}</span>",w));}else{o.push_str(w);}cur.clear();}o.push(c);i+=1;}
if !cur.is_empty(){let w=cur.as_str();if self.kw.contains(&w){o.push_str(&format!("<span class=\"keyword\">{}</span>",w));}else{o.push_str(w);}}o}
}
impl LanguageSupport for KotlinLang{fn name(&self)->&str{"Kotlin"}fn extensions(&self)->Vec<&str>{vec!["kt","kts"]}fn syntax_highlight(&self,c:&str)->String{c.lines().map(|l|self.hl(l)).collect::<Vec<_>>().join("\n")+"\n"}}
