import { createFileRoute } from "@tanstack/react-router";

import { AuthPage } from "@/components/auth/auth-page";
import { ForgotPasswordForm } from "@/components/auth/forgot-password-form";

export const Route = createFileRoute("/forgot-password")({
  component: ForgotPasswordPage,
});

function ForgotPasswordPage() {
  return (
    <AuthPage
      title="Forgot password"
      description="We'll email a reset link if that address is registered."
    >
      <ForgotPasswordForm />
    </AuthPage>
  );
}
