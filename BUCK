cxx_binary(
    name = "cargo_wrapper",
    srcs = ["tools/cargo_wrapper.c"],
)

command_alias(
    name = "buck_test_runner",
    exe = ":cargo_wrapper",
    args = ["run", "-p", "gunbc-test-runner", "--"],
)

sh_test(
    name = "buck_test",
    test = ":buck_test_runner",
    labels = ["buck2_run_from_project_root"],
)
