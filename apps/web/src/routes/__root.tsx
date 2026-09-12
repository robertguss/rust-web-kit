import type { QueryClient } from "@tanstack/react-query";
import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { ThemeProvider } from "next-themes";

import { ErrorFallback, NotFoundPage } from "@/components/error-fallback";
import { PendingScreen } from "@/components/pending-screen";
import { Toaster } from "@/components/ui/sonner";

export interface RouterContext {
  queryClient: QueryClient;
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootComponent,
  pendingComponent: PendingScreen,
  errorComponent: ErrorFallback,
  notFoundComponent: NotFoundPage,
});

function RootComponent() {
  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="system"
      enableSystem
      storageKey="rwk-theme"
      disableTransitionOnChange
    >
      <Outlet />
      <Toaster />
      {import.meta.env.DEV ? (
        <TanStackRouterDevtools position="bottom-right" />
      ) : null}
    </ThemeProvider>
  );
}
