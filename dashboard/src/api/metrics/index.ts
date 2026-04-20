import { request } from "@/api/client"
import type {
  CacheMetricRow,
  CacheMetricBucket,
  RequestMetricRow,
  RequestMetricBucket,
  RequestMetricSummary,
} from "@/lib/metrics"
import type {
  Pagination,
  BaseMetricQuery,
  AggregateCacheQuery,
  RequestMetricQuery,
  AggregateRequestQuery,
  SummaryQuery,
} from "@/lib/types"

export const metricsApi = {
  cache: {
    get: (props?: Pagination & BaseMetricQuery) =>
      request<CacheMetricRow[]>({
        path: "metrics/cache",
        query: props ?? {},
      }),
    aggregate: (props?: AggregateCacheQuery) =>
      request<CacheMetricBucket[]>({
        path: "metrics/cache/aggregate",
        query: props ?? {},
      }),
  },
  request: {
    get: (props?: Pagination & RequestMetricQuery) =>
      request<RequestMetricRow[]>({
        path: "metrics/requests",
        query: props ?? {},
      }),
    aggregate: (props?: AggregateRequestQuery) =>
      request<RequestMetricBucket[]>({
        path: "metrics/requests/aggregate",
        query: props ?? {},
      }),
    summary: (props?: SummaryQuery) =>
      request<RequestMetricSummary[]>({
        path: "metrics/requests/summary",
        query: props ?? {},
      }),
  },
}
