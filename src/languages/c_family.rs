// Phobos IDE - C/C++ Family Support
use crate::languages::LanguageSupport;
pub struct CFamilyLang {
    keywords: Vec<&'static str>,
    types: Vec<&'static str>,
}
impl CFamilyLang {
    pub fn new() -> Self {
        Self {
            keywords: vec!["auto","break","case","char","const","continue","default","do","double","else","enum","extern","float","for","goto","if","inline","int","long","register","restrict","return","short","signed","sizeof","static","struct","switch","typedef","union","unsigned","void","volatile","while","_Bool","_Complex","class","namespace","template","typename","public","private","protected","virtual","override","final","new","delete","try","catch","throw","using","constexpr","decltype","noexcept","nullptr","static_assert","thread_local","asm"],
            types: vec!["int8_t","int16_t","int32_t","int64_t","uint8_t","uint16_t","uint32_t","uint64_t","size_t","ssize_t","string","vector","map","set","unordered_map","shared_ptr","unique_ptr","optional"],
        }
    }
    fn hl(&self, line: &str) -> String {
        let mut out=String::new();let mut in_str=false;let mut cur=String::new();
        let ch:Vec<char>=line.chars().collect();let mut i=0;
        while i<ch.len(){let c=ch[i];
            if !in_str&&i+1<ch.len()&&ch[i]=='/'&&(ch[i+1]=='/'||ch[i+1]=='*'){out.push_str(&format!("<span class=\"comment\">{}</span>",&line[i..]));break;}
            if c=='"'&&(i==0||ch[i-1]!='\\'){in_str=!in_str;out.push(c);i+=1;continue;}
            if in_str{out.push(c);i+=1;continue;}
            if c.is_alphabetic()||c=='_'{cur.push(c);i+=1;continue;}
            if !cur.is_empty(){let w=cur.as_str();
                if self.keywords.contains(&w){out.push_str(&format!("<span class=\"keyword\">{}</span>",w));}
                else if self.types.contains(&w){out.push_str(&format!("<span class=\"type\">{}</span>",w));}
                else{out.push_str(w);}cur.clear();}
            out.push(c);i+=1;}
        if !cur.is_empty(){let w=cur.as_str();
            if self.keywords.contains(&w){out.push_str(&format!("<span class=\"keyword\">{}</span>",w));}else{out.push_str(w);}}
        out
    }
}
impl LanguageSupport for CFamilyLang {
    fn name(&self)->&str{"C/C++"}
    fn extensions(&self)->Vec<&str>{vec!["c","h","cpp","hpp","cc","cxx"]}
    fn syntax_highlight(&self,code:&str)->String{code.lines().map(|l|self.hl(l)).collect::<Vec<_>>().join("\n")+"\n"}
}
