import { QueryClientProvider } from "@tanstack/react-query";
import { createRouter, RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { installProblemInterceptor } from "@/api/client";
import { client } from "@/api/generated/client.gen";
import { createQueryClient } from "@/api/query";
import { ErrorFallback } from "@/components/error-fallback";
import { PendingScreen } from "@/components/pending-screen";
import { routeTree } from "@/routeTree.gen";
import "./index.css";

installProblemInterceptor(client);

const queryClient = createQueryClient();

const router = createRouter({
  routeTree,
  context: { queryClient },
  defaultPreload: "intent",
  defaultPendingComponent: PendingScreen,
  defaultErrorComponent: ErrorFallback,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const root = document.getElementById("root");
if (!root) {
  throw new Error("missing #root element");
}

createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </StrictMode>,
);
