use std::rc::Rc;
use v1_compiler::v1_compiler_compile::{compile_sources,SourceFile};
use v1_compiler::v1_compiler_artifact::RenderTarget;
fn main(){
for (label,early,fields) in [("baseline","","child: Present { value: item }, text: item.text"),("move_first","if early { return item }","child: Present { value: item }, text: item.text"),("borrow_first","if early { return item }","text: item.text, child: Present { value: item }")] {
let source=format!("module move_probe\ntype Item {{ child: Item? text: String other: String }}\nfn build(item: Item, early: Bool) -> Item {{ {early} if early {{ Item {{ {fields}, other: \"\" }} }} else {{ Item {{ child: item.child, text: item.text, other: item.other }} }} }}");
let r=compile_sources(Rc::new(im::vector![Rc::new(SourceFile{path:"move_probe.dag".into(),content:source})]),RenderTarget::Rust);
eprintln!("CASE {label} diagnostics={:?} ownership={:?}",r.diagnostics,r.ownership);
for (i,f) in r.files.iter().enumerate(){std::fs::write(format!("/tmp/crisp-seed-recovery/move-{label}-{i}.rs"),&f.content).unwrap();}
}
}
