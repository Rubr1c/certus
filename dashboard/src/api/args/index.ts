import { CmdArgs } from "@/lib/types";
import { request } from "../client";

export const argsApi = {
  get: () => request<CmdArgs>({ path: "args" }),
}
