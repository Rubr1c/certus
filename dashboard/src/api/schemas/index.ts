import { request } from "../client"
import type { ReqResSchema, Pagination } from "@/lib/types"

export const schemasApi = {
  get: (props?: Pagination) =>
    request<ReqResSchema[]>({
      path: "schemas",
      query: props ?? {},
    }),
}
