use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Input payload for adding a named attachment by content address.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AttachmentInput {
    pub name: String,
    pub cid: String,
}

/// A single attachment row returned from an ownable contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Attachment {
    pub name: String,
    pub cid: String,
}

/// Response payload for attachment listing queries.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct GetAttachmentsResponse {
    pub attachments: Vec<Attachment>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;
    use serde_json::json;

    #[test]
    fn attachment_input_serializes_with_locked_field_names() {
        let attachment = AttachmentInput {
            name: "passport.pdf".to_string(),
            cid: "bafybeihash".to_string(),
        };

        let value = serde_json::to_value(&attachment).expect("serialize attachment input");

        assert_eq!(
            value,
            json!({
                "name": "passport.pdf",
                "cid": "bafybeihash"
            })
        );
    }

    #[test]
    fn get_attachments_response_serializes_flat_attachment_rows() {
        let response = GetAttachmentsResponse {
            attachments: vec![
                Attachment {
                    name: "passport.pdf".to_string(),
                    cid: "bafybeiv1".to_string(),
                },
                Attachment {
                    name: "passport.pdf".to_string(),
                    cid: "bafybeiv2".to_string(),
                },
            ],
        };

        let value = serde_json::to_value(&response).expect("serialize attachments response");

        assert_eq!(
            value,
            json!({
                "attachments": [
                    { "name": "passport.pdf", "cid": "bafybeiv1" },
                    { "name": "passport.pdf", "cid": "bafybeiv2" }
                ]
            })
        );
    }

    #[test]
    fn get_attachments_response_schema_exposes_attachment_rows() {
        let schema = schema_for!(GetAttachmentsResponse);
        let root = serde_json::to_value(&schema).expect("schema to json");

        assert_eq!(
            root["properties"]["attachments"]["type"],
            serde_json::Value::String("array".to_string())
        );
        assert!(root["definitions"]["Attachment"]["properties"]["name"].is_object());
        assert!(root["definitions"]["Attachment"]["properties"]["cid"].is_object());
    }
}
