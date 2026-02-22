import re

with open("core/daglang/daglang-cli/tests/cli_commands.rs", "r") as f:
    text = f.read()

left = ["examples.rich_types", "infra.aws.services", "infra.azure.services", "infra.core", "infra.gcp.services", "infra.spec", "infra.aws.config", "infra.aws.resources", "infra.azure.config", "infra.azure.resources", "infra.gcp.config", "infra.gcp.resources", "examples.integration_tests", "infra.sdlc.deploy", "services.cargo", "services.gcp.iam", "services.gcp.secret_manager", "services.gcp.sts", "services.github.gist", "services.github.pull_request", "services.llm.anthropic", "services.llm.openai", "std.resources", "std.types", "cloud.aws.credential", "cloud.azure.credential", "examples.abstract_services", "interfaces.agent_provider", "interfaces.artifact_store", "interfaces.claim_store", "interfaces.issue_provider", "funcs.test_control_flow", "interfaces.outcome_ledger", "interfaces.signal_store", "profiles.cloud_run", "profiles.local", "profiles.sdlc", "profiles.unit_test", "services.git", "services.github.issues", "services.sdlc.providers.codex_agent_provider", "services.sdlc.providers.file_claim_store", "services.sdlc.providers.file_outcome_ledger", "services.sdlc.providers.gcs_claim_store", "services.sdlc.providers.gcs_outcome_ledger", "services.sdlc.providers.github_issue_provider", "services.sdlc.providers.stub_providers", "services.shell", "std.patterns", "cloud.gcp.credential", "shared.dag_util", "shared.gist_modes", "std.state_machines", "pipelines.reconciler", "tools.bootstrap", "tools.build", "tools.clippy", "tools.codegen", "tools.compilation", "tools.dag_viz", "tools.deps", "tools.design", "funcs.sdlc_stages", "funcs.sdlc_worker", "pipelines.sdlc", "tools.docgen", "tools.gist", "examples.deployment", "tools.infra", "tools.makegen", "tools.pragma", "tools.review", "tools.testgen", "pipelines.ci", "workflows.build_all", "workflows.makegen"]

# replace expected_real_corpus_module_order
block1 = "vec![\n" + "".join([f'        "{m}",\n' for m in left]) + "    ]"
text = re.sub(r'fn expected_real_corpus_module_order\(\) -> Vec<&\'static str> \{\n    vec\!\[.*?\]\n\}', f'fn expected_real_corpus_module_order() -> Vec<&\'static str> {{\n    {block1}\n}}', text, flags=re.DOTALL)

# replace expected_dsl_modules_sorted
sorted_left = sorted(left)
block2 = "vec![\n" + "".join([f'        "{m}",\n' for m in sorted_left]) + "    ]"
text = re.sub(r'fn expected_dsl_modules_sorted\(\) -> Vec<&\'static str> \{\n    vec\!\[.*?\]\n\}', f'fn expected_dsl_modules_sorted() -> Vec<&\'static str> {{\n    {block2}\n}}', text, flags=re.DOTALL)

with open("core/daglang/daglang-cli/tests/cli_commands.rs", "w") as f:
    f.write(text)
