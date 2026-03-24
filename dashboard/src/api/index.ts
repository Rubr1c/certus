import { argsApi } from "./args"
import { configApi } from "./config"
import { logsApi } from "./logs"
import { metricsApi } from "./metrics"
import { routesApi } from "./routes"

export const api = {
  config: configApi,
  logs: logsApi,
  metrics: metricsApi,
  args: argsApi,
  routes: routesApi,
}
