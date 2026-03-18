import { request } from "../client"
import type { ReqResSchema } from "@/lib/schema"
import type { Pagination } from "@/lib/query"

export const schemasApi = {
  get: (props?: Pagination) =>
    request<ReqResSchema[]>({
      path: "schemas",
      query: props ?? {},
    }),
}
