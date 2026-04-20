import { parseAsString, useQueryStates } from "nuqs";
import type { BaseMetricQuery, Interval } from "@/lib/types";

const metricFilterSchema = {
  from: parseAsString.withDefault(""),
  to: parseAsString.withDefault(""),
  interval: parseAsString.withDefault("15m"), 
};

export function useMetricFilter() {
  const [filter, setFilter] = useQueryStates(metricFilterSchema, {
    shallow: false,
    history: "push",
  });

  const updateFilter = (updates: Partial<typeof filter>) => {
    setFilter(updates);
  };

  const queryParams: BaseMetricQuery & { interval?: Interval } = {};
  if (filter.from) queryParams.from = filter.from;
  if (filter.to) queryParams.to = filter.to;
  if (filter.interval) queryParams.interval = filter.interval as Interval;

  return {
    filter,
    updateFilter,
    queryParams,
  };
}
