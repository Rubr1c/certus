
export const WEB_SOCKET_TYPES = {
  Logs: "logs",
  Metrics: "metrics",
} as const;

export type WebSocketType = (typeof WEB_SOCKET_TYPES)[keyof typeof WEB_SOCKET_TYPES];


export interface CmdArgs {
  ws: WebSocketType[],
}
