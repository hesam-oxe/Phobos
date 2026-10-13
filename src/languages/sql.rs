// Phobos IDE - SQL Support
use crate::languages::LanguageSupport;
pub struct SqlLang{kw:Vec<&'static str>}
impl SqlLang{pub fn new()->Self{Self{kw:vec!["SELECT","FROM","WHERE","INSERT","INTO","VALUES","UPDATE","SET","DELETE","CREATE","TABLE","ALTER","DROP","INDEX","VIEW","JOIN","LEFT","RIGHT","INNER","OUTER","FULL","ON","AS","AND","OR","NOT","NULL","PRIMARY","KEY","FOREIGN","REFERENCES","UNIQUE","CHECK","DEFAULT","CONSTRAINT","ORDER","BY","GROUP","HAVING","LIMIT","OFFSET","UNION","ALL","DISTINCT","EXISTS","BETWEEN","LIKE","IN","IS","CASE","WHEN","THEN","ELSE","END","BEGIN","COMMIT","ROLLBACK","TRANSACTION","GRANT","REVOKE","TRIGGER","PROCEDURE","FUNCTION"]}}
fn hl(&self,l:&str)->String{let t=l.trim_start();if t.starts_with("--"){return format!("<span class=\"comment\">{}</span>",l);}
let mut o=String::new();let mut cur=String::new();
for c in l.chars(){if c.is_alphabetic()||c=='_'{cur.push(c);continue;}
if !cur.is_empty(){let up=cur.to_uppercase();if self.kw.contains(&up.as_str()){o.push_str(&format!("<span class=\"keyword\">{}</span>",cur));}else{o.push_str(&cur);}cur.clear();}o.push(c);}
if !cur.is_empty(){let up=cur.to_uppercase();if self.kw.contains(&up.as_str()){o.push_str(&format!("<span class=\"keyword\">{}</span>",cur));}else{o.push_str(&cur);}}o}}
impl LanguageSupport for SqlLang{fn name(&self)->&str{"SQL"}fn extensions(&self)->Vec<&str>{vec!["sql"]}fn syntax_highlight(&self,c:&str)->String{c.lines().map(|l|self.hl(l)).collect::<Vec<_>>().join("\n")+"\n"}}
