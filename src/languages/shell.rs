// Phobos IDE - Shell/Bash Support
use crate::languages::LanguageSupport;
pub struct ShellLang{kw:Vec<&'static str>}
impl ShellLang{pub fn new()->Self{Self{kw:vec!["if","then","else","elif","fi","case","esac","for","while","until","do","done","in","function","select","time","coproc","echo","printf","read","cd","pwd","pushd","popd","dirs","let","declare","typeset","export","readonly","local","unset","set","shift","exit","return","break","continue","trap","source","alias","unalias","test"]}}}
fn hl(&self,l:&str)->String{let t=l.trim_start();if t.starts_with('#'){return format!("<span class=\"comment\">{}</span>",l);}
let mut o=String::new();let mut cur=String::new();
for c in l.chars(){if c.is_alphanumeric()||c=='_'{cur.push(c);continue;}
if !cur.is_empty(){if self.kw.contains(&cur.as_str()){o.push_str(&format!("<span class=\"keyword\">{}</span>",cur));}else if cur.starts_with('$')||c=='$'{o.push_str(&format!("<span class=\"type\">{}</span>",cur));}else{o.push_str(&cur);}cur.clear();}
if c=='$'{cur.push(c);}else{o.push(c);}}
if !cur.is_empty(){o.push_str(&cur);}o}}
impl LanguageSupport for ShellLang{fn name(&self)->&str{"Shell"}fn extensions(&self)->Vec<&str>{vec!["sh","bash","zsh","fish"]}fn syntax_highlight(&self,c:&str)->String{c.lines().map(|l|self.hl(l)).collect::<Vec<_>>().join("\n")+"\n"}}
