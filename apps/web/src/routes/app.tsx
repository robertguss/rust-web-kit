import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";

import { fetchCurrentUser } from "@/api/query";
import { AppNav } from "@/components/app-nav";

export const Route = createFileRoute("/app")({
  beforeLoad: async ({ context, location }) => {
    const user = await fetchCurrentUser(context.queryClient);
    if (!user) {
      throw redirect({
        to: "/login",
        search: {
          redirect: `${location.pathname}${location.searchStr}`,
        },
      });
    }
    return { user };
  },
  component: AppLayout,
});

function AppLayout() {
  return (
    <div className="flex min-h-svh flex-col">
      <AppNav />
      <main className="mx-auto w-full max-w-3xl flex-1 p-6">
        <Outlet />
      </main>
    </div>
  );
}
