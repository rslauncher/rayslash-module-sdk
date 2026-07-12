#[allow(warnings)]
mod bindings;

use bindings::exports::rayslash::module::provider::Guest;
use bindings::rayslash::module::types::{
    Action, Icon, ModuleError, QueryContext, QueryResponse, ResultItem,
};

struct Component;

impl Guest for Component {
    fn query(context: QueryContext) -> Result<QueryResponse, ModuleError> {
        let Some(name) = context.query.trim().strip_prefix("hello") else {
            return Ok(QueryResponse {
                results: Vec::new(),
                exclusive: false,
            });
        };
        let name = name.trim();
        let greeting = if name.is_empty() {
            "Hello!".to_owned()
        } else {
            format!("Hello, {name}!")
        };
        Ok(QueryResponse {
            results: vec![ResultItem {
                id: format!("hello:{}", name.to_ascii_lowercase()),
                title: greeting.clone(),
                subtitle: "Press Enter to copy".into(),
                icon: Icon::Text("H".into()),
                score: None,
                action: Action::CopyText(greeting),
            }],
            exclusive: false,
        })
    }
}

bindings::export!(Component with_types_in bindings);
