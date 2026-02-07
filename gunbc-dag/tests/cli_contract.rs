use gunbc_dag::makegen::ToolRegistry;
use gunbc_dag::render_makefile;

#[test]
fn test_makefile_cli_args_match_entrypoints() {
    let registry = ToolRegistry::default_registry();
    let makefile = render_makefile(&registry);

    for tool in &registry.tools {
        for param in &tool.entrypoints {
            let expected = if param.repeatable {
                format!(
                    "$(if $({}),$(foreach v,$({}),{} $(v)))",
                    param.make_var, param.make_var, param.cli_flag
                )
            } else {
                format!(
                    "$(if $({}),{} $({}))",
                    param.make_var, param.cli_flag, param.make_var
                )
            };

            assert!(
                makefile.contains(&expected),
                "Makefile missing CLI arg wiring for {}.{}: expected `{}`",
                tool.short_name,
                param.port_name,
                expected
            );
        }
    }
}

#[test]
fn test_makefile_help_repeatable_params() {
    let registry = ToolRegistry::default_registry();
    let makefile = render_makefile(&registry);

    for tool in &registry.tools {
        for param in &tool.entrypoints {
            if !param.repeatable {
                continue;
            }

            let expected = match &param.default {
                Some(default) => format!("[{}={} ...]", param.make_var, default),
                None => format!("[{}=... ...]", param.make_var),
            };

            assert!(
                makefile.contains(&expected),
                "Makefile help missing repeatable param for {}.{}: expected `{}`",
                tool.short_name,
                param.port_name,
                expected
            );
        }
    }
}
