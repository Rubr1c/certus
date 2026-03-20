import { argsApi } from "./args"
import { configApi } from "./config"
import { logsApi } from "./logs"
import { metricsApi } from "./metrics"
import { schemasApi } from "./schemas"

export const api = {
  config: configApi,
  logs: logsApi,
  metrics: metricsApi,
  schemas: schemasApi,
  args: argsApi,
}
