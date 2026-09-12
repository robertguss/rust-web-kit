import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { toast } from "sonner";

import {
  deleteProjectMutation,
  listProjectsQueryKey,
} from "@/api/generated/@tanstack/react-query.gen";
import { ProblemError } from "@/api/problem";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";

export function DeleteProjectDialog({
  id,
  name,
}: {
  id: string;
  name: string;
}) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const del = useMutation(deleteProjectMutation());

  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button type="button" variant="destructive">
          Delete
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete {name}?</AlertDialogTitle>
          <AlertDialogDescription>
            This cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            disabled={del.isPending}
            onClick={() => {
              void del
                .mutateAsync({ path: { id } })
                .then(async () => {
                  await queryClient.invalidateQueries({
                    queryKey: listProjectsQueryKey(),
                  });
                  toast.success("Project deleted");
                  await navigate({ to: "/app/projects" });
                })
                .catch((error: unknown) => {
                  toast.error(
                    error instanceof ProblemError
                      ? error.detail
                      : "Could not delete project",
                  );
                });
            }}
          >
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
