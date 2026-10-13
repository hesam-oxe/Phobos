// Phobos IDE - Dockerfile Support
use crate::languages::LanguageSupport;
pub struct DockerLang{kw:Vec<&'static str>}
impl DockerLang{pub fn new()->Self{Self{kw:vec!["FROM","RUN","CMD","LABEL","EXPOSE","ENV","ADD","COPY","ENTRYPOINT","VOLUME","USER","WORKDIR","ARG","ONBUILD","STOPSIGNAL","HEALTHCHECK","SHELL"]}}
fn hl(&self,l:&str)->String{let t=l.trim_start();if t.starts_with('#'){return format!("<span class=\"comment\">{}</span>",l);}
let mut o=String::new();let mut first=true;
for w in l.split_whitespace(){if first{let up=w.to_uppercase();if self.kw.contains(&up.as_str()){o.push_str(&format!("<span class=\"keyword\">{}</span>",w));}else{o.push_str(w);}first=false;}else{o.push(' ');o.push_str(w);}}o}
}
impl LanguageSupport for DockerLang{fn name(&self)->&str{"Docker"}fn extensions(&self)->Vec<&str>{vec!["dockerfile","Dockerfile"]}fn syntax_highlight(&self,c:&str)->String{c.lines().map(|l|self.hl(l)).collect::<Vec<_>>().join("\n")+"\n"}}
