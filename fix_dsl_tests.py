import os

path = "core/ir/src/platform.rs"
with open(path, "r") as f: content = f.read()

content = content.replace(
"""    #[test]
    fn os_dsl_platform_adapter_accepts_legacy_and_current_spellings() {
        assert_eq!(Os::parse_dsl_platform("Linux"), Os::Linux);
        assert_eq!(Os::parse_dsl_platform("Macos"), Os::Macos);
        assert_eq!(Os::parse_dsl_platform("MacOS"), Os::Macos);
        assert_eq!(Os::parse_dsl_platform("Windows"), Os::Windows);
    }""",
"""    #[test]
    fn os_dsl_platform_adapter_accepts_legacy_and_current_spellings() {
        assert_eq!(Os::parse_dsl_platform("Linux").unwrap(), Os::Linux);
        assert_eq!(Os::parse_dsl_platform("Macos").unwrap(), Os::Macos);
        assert_eq!(Os::parse_dsl_platform("MacOS").unwrap(), Os::Macos);
        assert_eq!(Os::parse_dsl_platform("Windows").unwrap(), Os::Windows);
    }"""
)

with open(path, "w") as f: f.write(content)
