import { createFileRoute, redirect } from "@tanstack/react-router";

import { fetchCurrentUser } from "@/api/query";
import { AuthPage } from "@/components/auth/auth-page";
import { LoginForm } from "@/components/auth/login-form";
import { safeReturnTo } from "@/lib/return-to";

type LoginSearch = {
  redirect?: string;
};

export const Route = createFileRoute("/login")({
  validateSearch: (search: Record<string, unknown>): LoginSearch => ({
    redirect: typeof search.redirect === "string" ? search.redirect : undefined,
  }),
  beforeLoad: async ({ context, search }) => {
    const user = await fetchCurrentUser(context.queryClient);
    if (user) {
      throw redirect({ href: safeReturnTo(search.redirect) });
    }
  },
  component: LoginPage,
});

function LoginPage() {
  const { redirect: redirectTo } = Route.useSearch();
  return (
    <AuthPage title="Log in" description="Sign in with email or OAuth.">
      <LoginForm redirect={redirectTo} />
    </AuthPage>
  );
}
