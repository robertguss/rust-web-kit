import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useForm } from "react-hook-form";
import { toast } from "sonner";

import { applyProblem } from "@/api/form-errors";
import { resetPasswordMutation } from "@/api/generated/@tanstack/react-query.gen";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { type ResetPasswordValues, resetPasswordSchema } from "@/lib/schemas";

export function ResetPasswordForm({ token }: { token: string }) {
  const navigate = useNavigate();
  const form = useForm<ResetPasswordValues>({
    resolver: zodResolver(resetPasswordSchema),
    defaultValues: { password: "" },
  });
  const reset = useMutation(resetPasswordMutation());

  const onSubmit = form.handleSubmit(async (values) => {
    try {
      await reset.mutateAsync({ body: { token, password: values.password } });
      toast.success("Password updated");
      await navigate({ to: "/login" });
    } catch (error) {
      applyProblem(error, form.setError);
    }
  });

  return (
    <form onSubmit={onSubmit} noValidate className="flex flex-col gap-4">
      <FieldGroup>
        <Field data-invalid={!!form.formState.errors.password}>
          <FieldLabel htmlFor="password">New password</FieldLabel>
          <Input
            id="password"
            type="password"
            autoComplete="new-password"
            aria-invalid={!!form.formState.errors.password}
            {...form.register("password")}
          />
          <FieldError>{form.formState.errors.password?.message}</FieldError>
        </Field>
      </FieldGroup>
      {form.formState.errors.root?.message ? (
        <FieldError>{form.formState.errors.root.message}</FieldError>
      ) : null}
      <Button type="submit" disabled={reset.isPending}>
        {reset.isPending ? "Saving…" : "Reset password"}
      </Button>
    </form>
  );
}
