import { createFileRoute, redirect } from "@tanstack/react-router";

import { fetchCurrentUser } from "@/api/query";
import { AuthPage } from "@/components/auth/auth-page";
import { LoginForm } from "@/components/auth/login-form";
import { safeReturnTo } from "@/lib/return-to";

type LoginSearch = {
  redirect?: string;
  error?: string;
};

function oauthErrorMessage(code: string): string {
  switch (code) {
    case "state":
      return "That sign-in attempt expired. Try again.";
    case "provider":
      return "Sign-in was cancelled or denied.";
    case "exchange":
      return "Could not complete sign-in. Try again.";
    case "conflict":
      return "An account with that email already exists. Sign in with your password.";
    default:
      return "Could not complete sign-in. Try again.";
  }
}

export const Route = createFileRoute("/login")({
  validateSearch: (search: Record<string, unknown>): LoginSearch => ({
    redirect: typeof search.redirect === "string" ? search.redirect : undefined,
    error: typeof search.error === "string" ? search.error : undefined,
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
  const { redirect: redirectTo, error } = Route.useSearch();
  return (
    <AuthPage title="Log in" description="Sign in with email or OAuth.">
      <div className="flex flex-col gap-4">
        {error ? (
          <p className="text-destructive text-sm" role="alert">
            {oauthErrorMessage(error)}
          </p>
        ) : null}
        <LoginForm redirect={redirectTo} />
      </div>
    </AuthPage>
  );
}
