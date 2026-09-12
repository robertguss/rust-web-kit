import { createFileRoute, redirect } from "@tanstack/react-router";

import { fetchCurrentUser } from "@/api/query";
import { AuthPage } from "@/components/auth/auth-page";
import { RegisterForm } from "@/components/auth/register-form";

export const Route = createFileRoute("/register")({
  beforeLoad: async ({ context }) => {
    const user = await fetchCurrentUser(context.queryClient);
    if (user) {
      throw redirect({ to: "/app/projects" });
    }
  },
  component: RegisterPage,
});

function RegisterPage() {
  return (
    <AuthPage
      title="Create an account"
      description="Email and password, or continue with OAuth."
    >
      <RegisterForm />
    </AuthPage>
  );
}
