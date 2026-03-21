import { api } from "@/api";
import { useQuery } from "@tanstack/react-query";

export function useArgs() {
  return useQuery({
    queryKey: ['args'],
    queryFn: api.args.get,
    staleTime: Infinity,
  });
}
