import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { useForm } from "react-hook-form";
import { toast } from "sonner";

import { applyProblem } from "@/api/form-errors";
import { registerMutation } from "@/api/generated/@tanstack/react-query.gen";
import { meQueryKey } from "@/api/query";
import { OauthButtons } from "@/components/oauth-buttons";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldError,
  FieldGroup,
  FieldLabel,
  FieldSeparator,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { type CredentialsValues, credentialsSchema } from "@/lib/schemas";

export function RegisterForm() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const form = useForm<CredentialsValues>({
    resolver: zodResolver(credentialsSchema),
    defaultValues: { email: "", password: "" },
  });
  const registerUser = useMutation(registerMutation());

  const onSubmit = form.handleSubmit(async (values) => {
    try {
      const user = await registerUser.mutateAsync({ body: values });
      queryClient.setQueryData(meQueryKey(), user);
      toast.success("Account created. Check your email to verify.");
      await navigate({ to: "/app/projects" });
    } catch (error) {
      applyProblem(error, form.setError);
    }
  });

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
        <Field data-invalid={!!form.formState.errors.password}>
          <FieldLabel htmlFor="password">Password</FieldLabel>
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
      <Button type="submit" disabled={registerUser.isPending}>
        {registerUser.isPending ? "Creating…" : "Create account"}
      </Button>
      <FieldSeparator>or</FieldSeparator>
      <OauthButtons />
      <p className="text-muted-foreground text-sm">
        Already have an account?{" "}
        <Link to="/login" className="underline underline-offset-4">
          Log in
        </Link>
      </p>
    </form>
  );
}
