import { type HttpMethod } from "@/lib/types"

const BASE_URL = "http://localhost:8080/_certus/api/v1"

interface RequestProps<TBody, TQuery> {
  path: string
  method?: HttpMethod
  body?: TBody
  query?: TQuery
}

type Query = Record<string, string | number | boolean | null | undefined>;

function toSearchParams(query: Query): URLSearchParams {
  const params = new URLSearchParams()
  for (const [k, v] of Object.entries(query)) {
    if (v !== undefined && v !== null) {
      params.set(k, String(v))
    }
  }
  return params
}

export async function request<
  TResponse,
  TBody = undefined,
  TQuery extends Record<string, any> = Record<string, any>
>(
  props: RequestProps<TBody, TQuery>
): Promise<TResponse> {
  let url = `${BASE_URL}/${props.path}`

  if (props.query && Object.keys(props.query).length > 0) {
    const params = toSearchParams(props.query)
    const search = params.toString()
    if (search) url += `?${search}`
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

  const text = await res.text()
  if (!text) {
    return null as TResponse
  }

  return JSON.parse(text) as TResponse
}
