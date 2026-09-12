import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useState } from "react";
import { useForm } from "react-hook-form";

import { applyProblem } from "@/api/form-errors";
import { forgotPasswordMutation } from "@/api/generated/@tanstack/react-query.gen";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { type EmailValues, emailSchema } from "@/lib/schemas";

export function ForgotPasswordForm() {
  const [sent, setSent] = useState(false);
  const form = useForm<EmailValues>({
    resolver: zodResolver(emailSchema),
    defaultValues: { email: "" },
  });
  const forgot = useMutation(forgotPasswordMutation());

  const onSubmit = form.handleSubmit(async (values) => {
    try {
      await forgot.mutateAsync({ body: values });
      setSent(true);
    } catch (error) {
      applyProblem(error, form.setError);
    }
  });

  if (sent) {
    return (
      <p className="text-sm">
        If that email is registered, a reset link is on its way.{" "}
        <Link to="/login" className="underline underline-offset-4">
          Back to log in
        </Link>
      </p>
    );
  }

  return (
    <form onSubmit={onSubmit} noValidate className="flex flex-col gap-4">
      <FieldGroup>
        <Field data-invalid={!!form.formState.errors.email}>
          <FieldLabel htmlFor="email">Email</FieldLabel>
          <Input
            id="email"
            type="email"
            autoComplete="email"
            aria-invalid={!!form.formState.errors.email}
            {...form.register("email")}
          />
          <FieldError>{form.formState.errors.email?.message}</FieldError>
        </Field>
      </FieldGroup>
      {form.formState.errors.root?.message ? (
        <FieldError>{form.formState.errors.root.message}</FieldError>
      ) : null}
      <Button type="submit" disabled={forgot.isPending}>
        {forgot.isPending ? "Sending…" : "Send reset link"}
      </Button>
    </form>
  );
}
