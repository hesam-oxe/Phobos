// Phobos IDE - Java Language Support
use crate::languages::LanguageSupport;
pub struct JavaLang {
    keywords: Vec<&'static str>,
    types: Vec<&'static str>,
}
impl JavaLang {
    pub fn new() -> Self {
        Self {
            keywords: vec!["package","import","public","private","protected","static","final","abstract","class","interface","enum","extends","implements","new","return","if","else","switch","case","default","break","continue","for","while","do","try","catch","finally","throw","throws","this","super","void","synchronized","volatile","transient","native","strictfp","assert","instanceof"],
            types: vec!["int","long","short","byte","float","double","boolean","char","String","Object","List","Map","Set","ArrayList","HashMap","Optional","Stream","Integer","Long","Double","Boolean"],
        }
    }
    fn hl(&self, line: &str) -> String {
        let mut out = String::new(); let mut in_str=false; let mut cur=String::new();
        let ch: Vec<char>=line.chars().collect(); let mut i=0;
        while i<ch.len() {
            let c=ch[i];
            if !in_str && i+1<ch.len() && ch[i]=='/' && ch[i+1]=='/' { out.push_str(&format!("<span class=\"comment\">{}</span>",&line[i..])); break; }
            if c=='"'&&(i==0||ch[i-1]!='\\'){in_str=!in_str;out.push(c);i+=1;continue;}
            if in_str{out.push(c);i+=1;continue;}
            if c.is_alphabetic()||c=='_'{cur.push(c);i+=1;continue;}
            if !cur.is_empty(){let w=cur.as_str();
                if self.keywords.contains(&w){out.push_str(&format!("<span class=\"keyword\">{}</span>",w));}
                else if self.types.contains(&w){out.push_str(&format!("<span class=\"type\">{}</span>",w));}
                else{out.push_str(w);}cur.clear();}
            out.push(c);i+=1;
        }
        if !cur.is_empty(){let w=cur.as_str();
            if self.keywords.contains(&w){out.push_str(&format!("<span class=\"keyword\">{}</span>",w));}else{out.push_str(w);}}
        out
    }
}
impl LanguageSupport for JavaLang {
    fn name(&self)->&str{"Java"}
    fn extensions(&self)->Vec<&str>{vec!["java"]}
    fn syntax_highlight(&self,code:&str)->String{code.lines().map(|l|self.hl(l)).collect::<Vec<_>>().join("\n")+"\n"}
}
