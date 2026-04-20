use crate::{ai::error::AiError, schema::ReqResSchema};

#[inline(always)]
pub fn build(records: &[ReqResSchema]) -> Result<String, AiError> {
    let json = serde_json::to_string_pretty(records)?;

    Ok(format!(
        r"You document HTTP traffic observed by an API gateway. You receive JSON records with fields: full_path, method, query_params, status_code, has_auth, req_headers, res_headers, body_schema (optional shallow JSON shape of response body when captured).

Rules:
- Describe ONLY what the records show. Do not invent routes, parameters, or response fields.
- When body_schema is null, say the response body was not captured (uncached traffic or non-JSON).
- Treat has_auth as an observation that auth-related headers were present, not as documentation of token format.
- req_headers/res_headers are filtered redacted JSON maps.

Output constraints:
- Return valid JSON only.
- Keep explanations concise and readable in docs UI.
- Prefer short bullet-like observations in arrays rather than long paragraphs.
- The limitations field for each endpoint must be an array of strings.

Records JSON:
{json}"
    ))
}
