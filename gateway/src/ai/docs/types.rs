use serde::{Deserialize, Serialize};

fn deserialize_string_list<'de, D>(
    deserializer: D,
) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        One(String),
        Many(Vec<String>),
    }

    let value = Option::<StringOrVec>::deserialize(deserializer)?;
    Ok(match value {
        Some(StringOrVec::One(s)) => {
            if s.trim().is_empty() {
                Vec::new()
            } else {
                vec![s]
            }
        }
        Some(StringOrVec::Many(v)) => v,
        None => Vec::new(),
    })
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GeneratedApiDocs {
    pub title: String,
    pub introduction: String,
    #[serde(default, deserialize_with = "deserialize_string_list")]
    pub highlights: Vec<String>,
    pub endpoints: Vec<EndpointDoc>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EndpointDoc {
    pub path: String,
    pub method: String,
    pub status_code: u16,
    pub title: String,
    pub summary: String,
    #[serde(default, deserialize_with = "deserialize_string_list")]
    pub observed_request: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_list")]
    pub observed_response: Vec<String>,
    #[serde(default)]
    pub markdown: String,
    #[serde(default, deserialize_with = "deserialize_string_list")]
    pub limitations: Vec<String>,
}
