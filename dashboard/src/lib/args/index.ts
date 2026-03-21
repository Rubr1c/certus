
export const WEB_SOCKET_TYPE = {
  Logs: "Logs",
  Metrics: "Metrics",
} as const;

export type WebSocketType = (typeof WEB_SOCKET_TYPE)[keyof typeof WEB_SOCKET_TYPE];


export interface CmdArgs {
  ws: WebSocketType[],
}
