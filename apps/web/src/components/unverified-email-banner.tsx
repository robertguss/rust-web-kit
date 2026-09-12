import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import { toast } from "sonner";

import { resendVerificationMutation } from "@/api/generated/@tanstack/react-query.gen";
import { ProblemError } from "@/api/problem";
import { Button } from "@/components/ui/button";

const DISMISS_KEY = "rwk:unverified-email-banner-dismissed";

function readDismissed(): boolean {
  try {
    return sessionStorage.getItem(DISMISS_KEY) === "1";
  } catch {
    return false;
  }
}

function persistDismissed(): void {
  try {
    sessionStorage.setItem(DISMISS_KEY, "1");
  } catch {
    // sessionStorage can throw in private mode
  }
}

export function UnverifiedEmailBanner({
  email,
  emailVerified,
}: {
  email: string;
  emailVerified: boolean;
}) {
  const [dismissed, setDismissed] = useState(readDismissed);
  const resend = useMutation({
    ...resendVerificationMutation(),
    onSuccess: () => {
      toast.success("Verification email sent");
    },
    onError: (error) => {
      const message =
        error instanceof ProblemError
          ? error.detail
          : "Could not send verification email.";
      toast.error(message);
    },
  });

  if (emailVerified || dismissed) {
    return null;
  }

  return (
    <div
      role="status"
      className="border-border bg-muted/40 flex flex-col gap-3 border-b px-4 py-3 sm:flex-row sm:items-center sm:justify-between"
    >
      <p className="text-sm">
        <span className="font-medium">{email}</span> is not verified. Check your
        inbox, or resend the verification email.
      </p>
      <div className="flex items-center gap-2">
        <Button
          type="button"
          size="sm"
          disabled={resend.isPending}
          onClick={() => resend.mutate({})}
        >
          {resend.isPending ? "Sending…" : "Resend"}
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={() => {
            persistDismissed();
            setDismissed(true);
          }}
        >
          Dismiss
        </Button>
      </div>
    </div>
  );
}
