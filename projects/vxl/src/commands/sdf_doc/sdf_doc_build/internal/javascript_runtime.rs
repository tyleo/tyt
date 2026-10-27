use crate::CliValue;
use sdfj_builder::JavaScriptRuntime;

impl CliValue for JavaScriptRuntime {
    const VARIANTS: &'static [Self] = &[
        JavaScriptRuntime::Bun,
        JavaScriptRuntime::Deno,
        JavaScriptRuntime::Node,
    ];

    fn name(self) -> &'static str {
        self.program()
    }

    fn help(self) -> &'static str {
        match self {
            JavaScriptRuntime::Bun => "Run the model under Bun",
            JavaScriptRuntime::Deno => "Run the model under Deno",
            JavaScriptRuntime::Node => "Run the model under Node 24 or later",
        }
    }
}
