use std::process::Output;

pub trait ParseJsonOutput {
    /// Plucks the stdout of a command output and parses it into JSON, failing if not possible.
    fn parse_json(&self) -> serde_json::Value;

    /// Checks stdout to parse as JSON, with a custom error message on fail.
    fn parse_json_expect(&self, msg: &str) -> serde_json::Value;
}

impl ParseJsonOutput for Output {
    fn parse_json(&self) -> serde_json::Value {
        self.parse_json_expect("output should be JSON")
    }

    fn parse_json_expect(&self, msg: &str) -> serde_json::Value {
        serde_json::from_slice(&self.stdout).expect(msg)
    }
}
