import { argsApi } from "./args"
import { configApi } from "./config"
import { docsApi } from "./docs"
import { logsApi } from "./logs"
import { metricsApi } from "./metrics"
import { routesApi } from "./routes"

export const api = {
  config: configApi,
  docs: docsApi,
  logs: logsApi,
  metrics: metricsApi,
  args: argsApi,
  routes: routesApi,
}
