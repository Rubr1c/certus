/** Mirrors gateway/src/schema/types.rs ReqResSchema (API response shape) */

export interface ReqResSchema {
  full_path: string
  method: string
  query_params: string | null
  status_code: number
  has_auth: boolean
  req_headers: string
  res_headers: string
  body_schema: string | null
}
