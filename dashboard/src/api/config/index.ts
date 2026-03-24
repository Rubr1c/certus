import { Config } from "@/lib/config/index"
import { request } from "../client"

export const configApi = {
  get: () => request<Config>({ path: "config" }),
  update: (config: Config) => request<undefined, Config>({ 
    path: "config", 
    method: "PUT",
    body: config,
  })
}
