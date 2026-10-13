// Phobos IDE - C# Support
use crate::languages::LanguageSupport;
pub struct CSharpLang {
    keywords: Vec<&'static str>,
}
impl CSharpLang {
    pub fn new()->Self{Self{keywords:vec!["using","namespace","class","struct","interface","enum","public","private","protected","internal","static","readonly","const","void","var","new","return","if","else","switch","case","default","break","continue","for","foreach","while","do","try","catch","finally","throw","this","base","null","true","false","async","await","yield","get","set","add","remove","where","select","from","in","orderby","join","let","on","equals","by","into","group","is","as","typeof","nameof","checked","unchecked","fixed","lock","params","ref","out","in","abstract","sealed","virtual","override","extern","partial","record"]}}
    fn hl(&self,line:&str)->String{
        let mut out=String::new();let mut ins=false;let mut cur=String::new();
        let ch:Vec<char>=line.chars().collect();let mut i=0;
        while i<ch.len(){let c=ch[i];
            if !ins&&i+1<ch.len()&&ch[i]=='/'&&ch[i+1]=='/'{out.push_str(&format!("<span class=\"comment\">{}</span>",&line[i..]));break;}
            if c=='"'&&(i==0||ch[i-1]!='\\'){ins=!ins;out.push(c);i+=1;continue;}
            if ins{out.push(c);i+=1;continue;}
            if c.is_alphabetic()||c=='_'{cur.push(c);i+=1;continue;}
            if !cur.is_empty(){let w=cur.as_str();
                if self.keywords.contains(&w){out.push_str(&format!("<span class=\"keyword\">{}</span>",w));}else{out.push_str(w);}cur.clear();}
            out.push(c);i+=1;}
        if !cur.is_empty(){let w=cur.as_str();
            if self.keywords.contains(&w){out.push_str(&format!("<span class=\"keyword\">{}</span>",w));}else{out.push_str(w);}}
        out
    }
}
impl LanguageSupport for CSharpLang{
    fn name(&self)->&str{"C#"}
    fn extensions(&self)->Vec<&str>{vec!["cs"]}
    fn syntax_highlight(&self,code:&str)->String{code.lines().map(|l|self.hl(l)).collect::<Vec<_>>().join("\n")+"\n"}
}
