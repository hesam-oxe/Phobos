// Phobos IDE - Assembly Support
use crate::languages::LanguageSupport;
pub struct AssemblyLang {
    keywords: Vec<&'static str>,
    registers: Vec<&'static str>,
}
impl AssemblyLang {
    pub fn new()->Self{Self{
        keywords:vec!["mov","push","pop","add","sub","mul","div","inc","dec","cmp","jmp","je","jne","jg","jl","jge","jle","ja","jb","call","ret","leave","nop","int","syscall","section","global","extern","db","dw","dd","dq","resb","resw","resd","equ","times","org","bits","lea","xor","and","or","not","shl","shr","rol","ror","test","loop","rep","movsb","stosb","lodsb","pushf","popf","iret"],
        registers:vec!["eax","ebx","ecx","edx","esi","edi","esp","ebp","rax","rbx","rcx","rdx","rsi","rdi","rsp","rbp","r8","r9","r10","r11","r12","r13","r14","r15","ax","bx","cx","dx","al","bl","cl","dl","ah","bh","ch","dh","xmm0","xmm1","rip"],
    }}
    fn hl(&self,line:&str)->String{
        if line.trim_start().starts_with(';')||line.trim_start().starts_with('#'){return format!("<span class=\"comment\">{}</span>",line);}
        let mut out=String::new();let mut cur=String::new();
        for c in line.chars(){
            if c.is_alphanumeric()||c=='_'||c=='.'{cur.push(c);continue;}
            if !cur.is_empty(){let w=cur.as_str();let wl=w.to_lowercase();
                if self.keywords.contains(&wl.as_str()){out.push_str(&format!("<span class=\"keyword\">{}</span>",w));}
                else if self.registers.contains(&wl.as_str()){out.push_str(&format!("<span class=\"type\">{}</span>",w));}
                else{out.push_str(w);}cur.clear();}
            out.push(c);
        }
        if !cur.is_empty(){let w=cur.as_str();let wl=w.to_lowercase();
            if self.keywords.contains(&wl.as_str()){out.push_str(&format!("<span class=\"keyword\">{}</span>",w));}else{out.push_str(w);}}
        out
    }
}
impl LanguageSupport for AssemblyLang{
    fn name(&self)->&str{"Assembly"}
    fn extensions(&self)->Vec<&str>{vec!["asm","s","S","nasm"]}
    fn syntax_highlight(&self,code:&str)->String{code.lines().map(|l|self.hl(l)).collect::<Vec<_>>().join("\n")+"\n"}
}
