use crate::schema::types::ReqResSchema;

pub fn save_req_res_schemas(
    conn: &mut rusqlite::Connection,
    req_res_schemas: Vec<ReqResSchema>,
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    {
        let mut query = tx.prepare(
            "INSERT INTO req_res_schemas (full_path, method, query_params, status_code, has_auth, req_headers, res_headers, body_schema)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;

        for schema in req_res_schemas {
            query.execute(rusqlite::params![
                schema.full_path,
                schema.method,
                schema.query_params,
                schema.status_code,
                schema.has_auth,
                schema.req_headers,
                schema.res_headers,
                schema.body_schema,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn get_req_res_schemas(
    conn: &rusqlite::Connection,
    page: u32,
    page_size: u32,
) -> rusqlite::Result<Vec<ReqResSchema>> {
    let offset = page * page_size;

    let mut stmt = conn.prepare(
        "SELECT full_path, method, query_params, status_code, has_auth, req_headers, res_headers, body_schema
         FROM req_res_schemas
         ORDER BY id DESC
         LIMIT ?1 OFFSET ?2",
    )?;

    let rows = stmt.query_map(rusqlite::params![page_size, offset], |row| {
        Ok(ReqResSchema {
            full_path: row.get(0)?,
            method: row.get(1)?,
            query_params: row.get(2)?,
            status_code: row.get(3)?,
            has_auth: row.get(4)?,
            req_headers: row.get(5)?,
            res_headers: row.get(6)?,
            body_schema: row.get(7)?,
        })
    })?;

    rows.collect()
}
