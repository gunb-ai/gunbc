use crate::artifact::*;
use crate::rust_emit::*;
use crate::python_emit::*;
use crate::go_emit::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ReservedWordStrategy {
    PrefixEscape { prefix: String },
    SuffixEscape { suffix: String },
    #[default]
    NoEscape,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReservedWords {
    pub keywords: Rc<Vec<String>>,
    pub strategy: Rc<ReservedWordStrategy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectScaffold {
    pub manifest_file: Option<String>,
    pub module_init_file: Option<String>,
    pub source_file_extension: String,
    pub source_dir: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SerializationSpec {
    pub struct_derives: Option<String>,
    pub struct_derives_copy: Option<String>,
    pub enum_derives: Option<String>,
    pub enum_derives_copy: Option<String>,
    pub tag_attribute: Option<String>,
    pub rename_attribute_template: Option<String>,
    pub derive_attribute: Option<String>,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum TestNameStyle {
    #[default]
    SnakeCaseTestNames,
    PascalCaseTestNames,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TestConventions {
    pub file_prefix: String,
    pub file_suffix: String,
    pub file_dir: Option<String>,
    pub function_prefix: String,
    pub name_style: TestNameStyle,
    pub async_decorator: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LanguageSpec {
    pub target_name: String,
    pub reserved_words: Rc<ReservedWords>,
    pub scaffold: Rc<ProjectScaffold>,
    pub serialization: Rc<SerializationSpec>,
    pub test_conventions: Rc<TestConventions>,
    pub top_level_visibility: String,
}

pub fn rust_spec() -> Rc<LanguageSpec> {
    Rc::new(LanguageSpec { target_name: "rust".to_string(), reserved_words: Rc::new(ReservedWords { keywords: Rc::new(RUST_RESERVED.iter().map(|s| s.to_string()).collect::<Vec<_>>()), strategy: Rc::new(ReservedWordStrategy::PrefixEscape { prefix: RUST_RESERVED_ESCAPE_PREFIX.to_string() }) }), scaffold: Rc::new(ProjectScaffold { manifest_file: Some("Cargo.toml".to_string()), module_init_file: None, source_file_extension: RUST_SOURCE_EXTENSION.to_string(), source_dir: Some(RUST_SOURCE_DIR.to_string()) }), serialization: Rc::new(SerializationSpec { struct_derives: Some(RUST_STRUCT_DERIVES.to_string()), struct_derives_copy: Some(RUST_STRUCT_DERIVES_COPY.to_string()), enum_derives: Some(RUST_ENUM_DERIVES.to_string()), enum_derives_copy: Some(RUST_ENUM_DERIVES_COPY.to_string()), tag_attribute: Some(RUST_SERDE_TAG.to_string()), rename_attribute_template: Some(RUST_SERDE_RENAME_TEMPLATE.to_string()), derive_attribute: None, default_value: None }), test_conventions: Rc::new(TestConventions { file_prefix: "".to_string(), file_suffix: "_test".to_string(), file_dir: Some("tests/".to_string()), function_prefix: "test_".to_string(), name_style: TestNameStyle::SnakeCaseTestNames, async_decorator: Some("#[tokio::test]".to_string()) }), top_level_visibility: RUST_VISIBILITY.to_string() })
}

pub fn python_spec() -> Rc<LanguageSpec> {
    Rc::new(LanguageSpec { target_name: "python".to_string(), reserved_words: Rc::new(ReservedWords { keywords: Rc::new(PYTHON_RESERVED.iter().map(|s| s.to_string()).collect::<Vec<_>>()), strategy: Rc::new(ReservedWordStrategy::SuffixEscape { suffix: PYTHON_RESERVED_ESCAPE_SUFFIX.to_string() }) }), scaffold: Rc::new(ProjectScaffold { manifest_file: Some("requirements.txt".to_string()), module_init_file: Some(PYTHON_MODULE_INIT.to_string()), source_file_extension: PYTHON_SOURCE_EXTENSION.to_string(), source_dir: None }), serialization: Rc::new(SerializationSpec { struct_derives: None, struct_derives_copy: None, enum_derives: None, enum_derives_copy: None, tag_attribute: None, rename_attribute_template: None, derive_attribute: Some(PYTHON_DERIVE_ATTRIBUTE.to_string()), default_value: Some(PYTHON_DEFAULT_VALUE.to_string()) }), test_conventions: Rc::new(TestConventions { file_prefix: "test_".to_string(), file_suffix: "".to_string(), file_dir: Some("tests/".to_string()), function_prefix: "test_".to_string(), name_style: TestNameStyle::SnakeCaseTestNames, async_decorator: None }), top_level_visibility: "".to_string() })
}

pub fn go_spec() -> Rc<LanguageSpec> {
    Rc::new(LanguageSpec { target_name: "go".to_string(), reserved_words: Rc::new(ReservedWords { keywords: Rc::new(GO_RESERVED.iter().map(|s| s.to_string()).collect::<Vec<_>>()), strategy: Rc::new(ReservedWordStrategy::SuffixEscape { suffix: GO_RESERVED_ESCAPE_SUFFIX.to_string() }) }), scaffold: Rc::new(ProjectScaffold { manifest_file: Some(GO_MANIFEST_FILE.to_string()), module_init_file: None, source_file_extension: GO_SOURCE_EXTENSION.to_string(), source_dir: None }), serialization: Rc::new(SerializationSpec { struct_derives: None, struct_derives_copy: None, enum_derives: None, enum_derives_copy: None, tag_attribute: None, rename_attribute_template: None, derive_attribute: None, default_value: None }), test_conventions: Rc::new(TestConventions { file_prefix: "".to_string(), file_suffix: "_test".to_string(), file_dir: None, function_prefix: "Test".to_string(), name_style: TestNameStyle::PascalCaseTestNames, async_decorator: None }), top_level_visibility: "".to_string() })
}

pub fn language_spec_for_target(target: RenderTarget) -> Rc<LanguageSpec> {
    match target {
    RenderTarget::Rust => {
        rust_spec()
    }
    RenderTarget::Go => {
        go_spec()
    }
    RenderTarget::Python => {
        python_spec()
    }
    RenderTarget::Dag => {
        rust_spec()
    }
}
}

pub fn target_keyword(target: RenderTarget, key: &str) -> String {
    match target {
    RenderTarget::Rust => {
        match v2_rt::lookup(&RUST_KEYWORDS, key.to_string()) {
    Some(kw) => {
        kw.clone()
    }
    None => {
        key.to_string()
    }
}
    }
    RenderTarget::Go => {
        match v2_rt::lookup(&GO_KEYWORDS, key.to_string()) {
    Some(kw) => {
        kw.clone()
    }
    None => {
        key.to_string()
    }
}
    }
    RenderTarget::Python => {
        match v2_rt::lookup(&PYTHON_KEYWORDS, key.to_string()) {
    Some(kw) => {
        kw.clone()
    }
    None => {
        key.to_string()
    }
}
    }
    RenderTarget::Dag => {
        key.to_string()
    }
}
}

pub fn target_primitive_type(target: RenderTarget, name: &str) -> String {
    match target {
    RenderTarget::Rust => {
        match v2_rt::lookup(&RUST_TYPE_MAP, name.to_string()) {
    Some(mapped) => {
        mapped.clone()
    }
    None => {
        name.to_string()
    }
}
    }
    RenderTarget::Go => {
        match v2_rt::lookup(&GO_TYPE_MAP, name.to_string()) {
    Some(mapped) => {
        mapped.clone()
    }
    None => {
        name.to_string()
    }
}
    }
    RenderTarget::Python => {
        match v2_rt::lookup(&PYTHON_TYPE_MAP, name.to_string()) {
    Some(mapped) => {
        mapped.clone()
    }
    None => {
        name.to_string()
    }
}
    }
    RenderTarget::Dag => {
        name.to_string()
    }
}
}

pub fn target_container_template(target: RenderTarget, kind: &str) -> Option<String> {
    match target {
    RenderTarget::Rust => {
        v2_rt::lookup(&RUST_CONTAINER_TEMPLATES, kind.to_string())
    }
    RenderTarget::Go => {
        v2_rt::lookup(&GO_CONTAINER_TEMPLATES, kind.to_string())
    }
    RenderTarget::Python => {
        v2_rt::lookup(&PYTHON_CONTAINER_TEMPLATES, kind.to_string())
    }
    RenderTarget::Dag => {
        None
    }
}
}

pub fn scaffold_for_target(target: RenderTarget) -> Rc<ProjectScaffold> {
    language_spec_for_target(target).scaffold.clone()
}

pub fn serialization_for_target(target: RenderTarget) -> Rc<SerializationSpec> {
    language_spec_for_target(target).serialization.clone()
}

pub fn test_conventions_for_target(target: RenderTarget) -> Rc<TestConventions> {
    language_spec_for_target(target).test_conventions.clone()
}

pub fn top_level_visibility_for_target(target: RenderTarget) -> String {
    language_spec_for_target(target).top_level_visibility.clone()
}

