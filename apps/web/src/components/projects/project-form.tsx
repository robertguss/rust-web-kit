import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";

import { applyProblem } from "@/api/form-errors";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { type ProjectValues, projectSchema } from "@/lib/schemas";

export function ProjectForm({
  defaultValues,
  submitLabel,
  pending,
  onSubmit,
}: {
  defaultValues?: Partial<ProjectValues>;
  submitLabel: string;
  pending: boolean;
  onSubmit: (values: {
    name: string;
    description: string | null;
  }) => Promise<void>;
}) {
  const form = useForm<ProjectValues>({
    resolver: zodResolver(projectSchema),
    defaultValues: {
      name: defaultValues?.name ?? "",
      description: defaultValues?.description ?? "",
    },
  });

  const submit = form.handleSubmit(async (values) => {
    const description = values.description.trim();
    try {
      await onSubmit({
        name: values.name.trim(),
        description: description.length > 0 ? description : null,
      });
    } catch (error) {
      applyProblem(error, form.setError);
    }
  });

  return (
    <form onSubmit={submit} noValidate className="flex flex-col gap-4">
      <FieldGroup>
        <Field data-invalid={!!form.formState.errors.name}>
          <FieldLabel htmlFor="name">Name</FieldLabel>
          <Input
            id="name"
            aria-invalid={!!form.formState.errors.name}
            {...form.register("name")}
          />
          <FieldError>{form.formState.errors.name?.message}</FieldError>
        </Field>
        <Field data-invalid={!!form.formState.errors.description}>
          <FieldLabel htmlFor="description">Description</FieldLabel>
          <Textarea
            id="description"
            rows={4}
            aria-invalid={!!form.formState.errors.description}
            {...form.register("description")}
          />
          <FieldError>{form.formState.errors.description?.message}</FieldError>
        </Field>
      </FieldGroup>
      {form.formState.errors.root?.message ? (
        <FieldError>{form.formState.errors.root.message}</FieldError>
      ) : null}
      <Button type="submit" disabled={pending}>
        {pending ? "Saving…" : submitLabel}
      </Button>
    </form>
  );
}
