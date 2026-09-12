import { QueryClient, queryOptions } from "@tanstack/react-query";

import { meOptions, meQueryKey } from "./generated/@tanstack/react-query.gen";
import { ProblemError } from "./problem";

/** Browser QueryClient with no retries (401 must not loop). */
export function createQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        refetchOnWindowFocus: false,
        staleTime: 15_000,
      },
      mutations: {
        retry: false,
      },
    },
  });
}

/** Current-user query. 401 is a logged-out session, not a retryable failure. */
export function meQuery() {
  return queryOptions({
    ...meOptions(),
    retry: false,
    staleTime: 30_000,
  });
}

export { meQueryKey };

/** `null` when the session is missing (401). Other errors propagate. */
export async function fetchCurrentUser(queryClient: QueryClient) {
  try {
    return await queryClient.ensureQueryData(meQuery());
  } catch (error) {
    if (error instanceof ProblemError && error.status === 401) {
      return null;
    }
    throw error;
  }
}
