import { createFileRoute } from "@tanstack/react-router";

import { AuthPage } from "@/components/auth/auth-page";
import { ResetPasswordForm } from "@/components/auth/reset-password-form";

type ResetSearch = {
  token?: string;
};

export const Route = createFileRoute("/reset-password")({
  validateSearch: (search: Record<string, unknown>): ResetSearch => ({
    token: typeof search.token === "string" ? search.token : undefined,
  }),
  component: ResetPasswordPage,
});

function ResetPasswordPage() {
  const { token } = Route.useSearch();
  return (
    <AuthPage
      title="Reset password"
      description="Choose a new password for your account."
    >
      {token ? (
        <ResetPasswordForm token={token} />
      ) : (
        <p className="text-sm">This reset link is missing a token.</p>
      )}
    </AuthPage>
  );
}
