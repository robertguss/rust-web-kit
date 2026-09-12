import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { toast } from "sonner";

import {
  getProjectOptions,
  getProjectQueryKey,
  listProjectsQueryKey,
  updateProjectMutation,
} from "@/api/generated/@tanstack/react-query.gen";
import { DeleteProjectDialog } from "@/components/projects/delete-project-dialog";
import { ProjectForm } from "@/components/projects/project-form";
import { Skeleton } from "@/components/ui/skeleton";

export const Route = createFileRoute("/app/projects/$id")({
  component: ProjectDetailPage,
});

function ProjectDetailPage() {
  const { id } = Route.useParams();
  const queryClient = useQueryClient();
  const project = useQuery(getProjectOptions({ path: { id } }));
  const update = useMutation(updateProjectMutation());

  if (project.isPending) {
    return <Skeleton className="h-40 w-full" />;
  }

  if (project.isError || !project.data) {
    return <p className="text-destructive text-sm">Project not found.</p>;
  }

  const row = project.data;

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-start justify-between gap-3">
        <h1 className="font-heading text-xl font-medium">{row.name}</h1>
        <DeleteProjectDialog id={row.id} name={row.name} />
      </div>
      <ProjectForm
        key={row.updated_at}
        defaultValues={{
          name: row.name,
          description: row.description ?? "",
        }}
        submitLabel="Save changes"
        pending={update.isPending}
        onSubmit={async (values) => {
          await update.mutateAsync({ path: { id: row.id }, body: values });
          await Promise.all([
            queryClient.invalidateQueries({
              queryKey: getProjectQueryKey({ path: { id: row.id } }),
            }),
            queryClient.invalidateQueries({
              queryKey: listProjectsQueryKey(),
            }),
          ]);
          toast.success("Project updated");
        }}
      />
    </div>
  );
}
