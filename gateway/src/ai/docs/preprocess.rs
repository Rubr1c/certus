use std::collections::HashMap;

use crate::schema::ReqResSchema;

const PROMPT_ROW_CAP: usize = 100;

#[inline(always)]
pub fn dedupe_keep_newest(rows: Vec<ReqResSchema>) -> Vec<ReqResSchema> {
    let mut map: HashMap<(String, String, u16), ReqResSchema> = HashMap::new();
    for row in rows {
        let key = (row.full_path.clone(), row.method.clone(), row.status_code);
        map.entry(key).or_insert(row);
    }

    let mut deduped: Vec<_> = map.into_values().collect();
    deduped.sort_by(|a, b| {
        a.full_path
            .cmp(&b.full_path)
            .then_with(|| a.method.cmp(&b.method))
            .then_with(|| a.status_code.cmp(&b.status_code))
    });

    if deduped.len() > PROMPT_ROW_CAP {
        deduped.truncate(PROMPT_ROW_CAP);
    }

    deduped
}
