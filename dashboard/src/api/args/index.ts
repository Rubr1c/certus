import { CmdArgs } from "@/lib/args";
import { request } from "../client";

export const argsApi = {
  get: () => request<CmdArgs>({ path: "args" }),
}
