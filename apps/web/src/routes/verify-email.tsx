import { useMutation } from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";
import { type ReactNode, useEffect, useRef } from "react";

import { verifyEmailMutation } from "@/api/generated/@tanstack/react-query.gen";
import { ProblemError } from "@/api/problem";
import { AuthPage } from "@/components/auth/auth-page";
import { Button } from "@/components/ui/button";

type VerifySearch = {
  token?: string;
};

export const Route = createFileRoute("/verify-email")({
  validateSearch: (search: Record<string, unknown>): VerifySearch => ({
    token: typeof search.token === "string" ? search.token : undefined,
  }),
  component: VerifyEmailPage,
});

function VerifyEmailPage() {
  const { token } = Route.useSearch();
  const started = useRef(false);
  const verify = useMutation(verifyEmailMutation());

  useEffect(() => {
    if (!token || started.current) {
      return;
    }
    started.current = true;
    verify.mutate({ body: { token } });
  }, [token, verify]);

  let body: ReactNode;
  if (!token) {
    body = (
      <p className="text-sm">This verification link is missing a token.</p>
    );
  } else if (verify.isPending || verify.isIdle) {
    body = <p className="text-sm">Confirming your email…</p>;
  } else if (verify.isSuccess) {
    body = (
      <div className="flex flex-col gap-3">
        <p className="text-sm">Email verified.</p>
        <Button asChild>
          <Link to="/app/projects">Continue</Link>
        </Button>
      </div>
    );
  } else {
    const message =
      verify.error instanceof ProblemError
        ? verify.error.detail
        : "This link is invalid or expired.";
    body = <p className="text-sm">{message}</p>;
  }

  return (
    <AuthPage
      title="Verify email"
      description="Confirm the address on your account."
    >
      {body}
    </AuthPage>
  );
}
