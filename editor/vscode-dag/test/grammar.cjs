const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { createRequire } = require("node:module");

// An optional argument supplies a directory containing installed test dependencies.
const load = process.argv[2]
    ? createRequire(path.join(path.resolve(process.argv[2]), "package.json"))
    : require;

const textmate = load("vscode-textmate");
const oniguruma = load("vscode-oniguruma");

const extensionRoot = path.resolve(__dirname, "..");
const repoRoot = path.resolve(extensionRoot, "../..");
const grammarPath = path.join(extensionRoot, "syntaxes/dag.tmLanguage.json");
const lexerPath = path.join(repoRoot, "src/v2/extdeps/languages/dag.dag");

async function main() {
    const wasm = fs.readFileSync(
        load.resolve("vscode-oniguruma/release/onig.wasm")
    );
    await oniguruma.loadWASM(
        wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength)
    );

    const registry = new textmate.Registry({
        onigLib: Promise.resolve({
            createOnigScanner: patterns => new oniguruma.OnigScanner(patterns),
            createOnigString: text => new oniguruma.OnigString(text)
        }),
        loadGrammar: async scope =>
            scope === "source.dag"
                ? textmate.parseRawGrammar(
                    fs.readFileSync(grammarPath, "utf8"),
                    grammarPath
                )
                : null
    });

    try {
        const grammar = await registry.loadGrammar("source.dag");
        assert.ok(grammar, "Grammar did not load");

        let checks = 0;
        let failures = 0;

        function check(
            name, lines, lineNumber, fragment, scope,
            expected = true, exactToken = false
        ) {
            checks++;
            let state = textmate.INITIAL;
            let tokens;

            for (let index = 0; index <= lineNumber; index++) {
                const result = grammar.tokenizeLine(lines[index], state);
                assert.ok(!result.stoppedEarly, `${name}: tokenization stopped early`);
                state = result.ruleStack;
                tokens = result.tokens;
            }

            const start = lines[lineNumber].indexOf(fragment);
            assert.ok(start >= 0, `${name}: missing test fragment`);
            const end = start + fragment.length;
            const selected = tokens.filter(
                token => token.startIndex < end && token.endIndex > start
            );

            const covered = selected.length > 0
                && selected[0].startIndex <= start
                && selected[selected.length - 1].endIndex >= end;

            const correctScopes = selected.every(
                token => token.scopes.includes(scope) === expected
            );

            const correctBoundary = !exactToken || (
                selected.length === 1
                && selected[0].startIndex === start
                && selected[0].endIndex === end
            );

            const passed = covered && correctScopes && correctBoundary;
            console.log(`${passed ? "PASS" : "FAIL"} ${name}`);

            if (!passed) {
                failures++;
                console.log(JSON.stringify(selected, null, 2));
            }
        }

        check("keyword", ["if true"], 0, "if", "keyword.other.dag");
        check("constant", ["if true"], 0, "true", "constant.language.dag");
        check("float", ["12.34"], 0, "12.34", "constant.numeric.dag");
        check("comment shields keywords", ["// if true"], 0, "if",
            "comment.line.double-slash.dag");
        check("string shields keywords", ['"if true"'], 0, "if",
            "keyword.other.dag", false);
        check("multiline string", ['"first', 'second"'], 1, "second",
            "string.quoted.double.dag");
        check("escaped quote stays inside string", ['"a\\" if"'], 0, "if",
            "keyword.other.dag", false);
        check("escaped brace stays inside string", ['"\\{if true}"'], 0, "if",
            "meta.interpolation.dag", false);
        check("interpolation contains code", ['"{if true}"'], 0, "if",
            "keyword.other.dag");
        check("nested braces", ['"{(Thing { value: true })}"'], 0, "true",
            "meta.braces.dag");
        check("returns to string after interpolation", ['"{if true} after"'],
            0, "after", "meta.interpolation.dag", false);
        check("nested string shields closing brace", ['"{("}")} after"'],
            0, "after", "meta.interpolation.dag", false);
        check("keyword suffix inside identifier", ["motif"], 0, "if",
            "keyword.other.dag", false);
        check("digits inside identifier", ["item12"], 0, "12",
            "constant.numeric.dag", false);

        for (const operator of [
            "->", "=>", "==", "!=", ">=", "<=", "||", "&&", "|>", "??",
            "+", "-", "*", "/", "%", "!", "^", "=", ":", "|", "<", ">", "?"
        ]) {
            check(`operator ${operator}`, [`a ${operator} b`], 0, operator,
                "keyword.operator.dag", true, true);
        }

        check("operator inside interpolation", ['"{a + b}"'], 0, "+",
            "keyword.operator.dag", true, true);
        check("comment shields operators", ["// a + b"], 0, "+",
            "keyword.operator.dag", false);
        check("string shields operators", ['"a + b"'], 0, "+",
            "keyword.operator.dag", false);
        check("identifier remains intact", ["item12"], 0, "item12",
            "variable.other.dag", true, true);
        check("keyword suffix remains intact", ["motif"], 0, "motif",
            "variable.other.dag", true, true);
        check("decimal point is numeric", ["12.34"], 0, ".",
            "constant.numeric.dag");

        // Read current literal keyword declarations independently of the emitter.
        // The DAG witness separately checks the evaluated lexer rule population.
        const lexerSource = fs.readFileSync(lexerPath, "utf8");
        const start = lexerSource.indexOf("fn dag_lex_rules_except_braces()");
        const end = lexerSource.indexOf("fn dag_lex_rules()", start);
        assert.ok(start >= 0 && end > start, "Cannot locate lexer rule declarations");

        const declarations = lexerSource.slice(start, end);
        const keywordCalls = [
            ...declarations.matchAll(
                /dag_keyword_lex_rule\(\s*token_class:\s*\^(\w+),\s*text:\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*\)/g
            )
        ];
        assert.ok(keywordCalls.length > 0, "No lexer keywords discovered");

        for (const [, tokenClass, keyword] of keywordCalls) {
            const scope = ["dag_token_kw_true", "dag_token_kw_false"]
                .includes(tokenClass)
                ? "constant.language.dag"
                : "keyword.other.dag";

            check(`lexer keyword ${keyword}`, [keyword], 0, keyword,
                scope, true, true);

            for (const identifier of [
                `${keyword}Suffix`,
                `prefix${keyword}`,
                `${keyword}12`
            ]) {
                check(`identifier ${identifier}`, [identifier], 0, identifier,
                    "variable.other.dag", true, true);
            }
        }

        console.log(`Lexer keyword declarations checked: ${keywordCalls.length}`);
        console.log(`${checks - failures}/${checks} checks passed`);
        process.exitCode = failures ? 1 : 0;
    } finally {
        registry.dispose();
    }
}

main().catch(error => {
    console.error(error);
    process.exitCode = 1;
});
