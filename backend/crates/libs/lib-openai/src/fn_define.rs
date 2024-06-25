use async_openai::types::FunctionObjectArgs;
use serde_json::json;

pub(crate) fn article_summary_to_markdown_fn_def() -> FunctionObjectArgs {
    FunctionObjectArgs::default()
        .name("article_summary_to_markdown")
        .description("summary article content to markdown format")
        .parameters(json!(
            {
                "type": "object",
                "properties": {
                    "markdown": {
                        "type": "string",
                        "description": "markdown format content"
                    }
                },
                "required": ["markdown"]
            }
        ))
        .to_owned()
}
