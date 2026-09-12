import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { toast } from "sonner";

import {
  createProjectMutation,
  listProjectsQueryKey,
} from "@/api/generated/@tanstack/react-query.gen";
import { ProjectForm } from "@/components/projects/project-form";

export const Route = createFileRoute("/app/projects/new")({
  component: NewProjectPage,
});

function NewProjectPage() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const create = useMutation(createProjectMutation());

  return (
    <div className="flex flex-col gap-4">
      <h1 className="font-heading text-xl font-medium">New project</h1>
      <ProjectForm
        submitLabel="Create project"
        pending={create.isPending}
        onSubmit={async (values) => {
          const project = await create.mutateAsync({ body: values });
          await queryClient.invalidateQueries({
            queryKey: listProjectsQueryKey(),
          });
          toast.success("Project created");
          await navigate({
            to: "/app/projects/$id",
            params: { id: project.id },
          });
        }}
      />
    </div>
  );
}
