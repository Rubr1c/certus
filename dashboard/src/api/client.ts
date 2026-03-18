import { type HttpMethod } from "@/lib/http/index"

const BASE_URL = "http://localhost:8080/_certus/api/v1"

interface RequestProps<TBody, TQuery = object> {
  path: string
  method?: HttpMethod
  body?: TBody
  query?: TQuery
}

function toSearchParams(query: object): URLSearchParams {
  const params = new URLSearchParams()
  for (const [k, v] of Object.entries(query)) {
    if (v !== undefined && v !== null) {
      params.set(k, String(v))
    }
  }
  return params
}

export async function request<
  TResponse, TBody = undefined, 
  TQuery extends object = object
>(
  props: RequestProps<TBody, TQuery>
): Promise<TResponse> {
  let url = `${BASE_URL}/${props.path}`

  if (props.query && Object.keys(props.query).length > 0) {
    const params = toSearchParams(props.query)
    if (params.toString()) url += `?${params.toString()}`
  }

  const res = await fetch(url, {
    method: props.method ?? "GET",
    headers:
      props.body !== undefined
        ? { "Content-Type": "application/json" }
        : undefined,
    body:
      props.body !== undefined ? JSON.stringify(props.body) : undefined,
  })

  if (!res.ok) {
    throw new Error(`Request failed: ${res.status} ${res.statusText}`)
  }

  return res.json() as Promise<TResponse>
}
