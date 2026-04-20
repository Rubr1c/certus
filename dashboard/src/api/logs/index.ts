import { request } from "../client"
import type { LogEntry } from "@/lib/log"
import type { Pagination, LogQuery } from "@/lib/types"

export const logsApi = {
  get: (props?: Pagination & LogQuery) =>
    request<LogEntry[]>({
      path: "logs",
      query: props ?? {},
    }),
}
