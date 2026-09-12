import { useQuery } from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";

import { listProjectsOptions } from "@/api/generated/@tanstack/react-query.gen";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";

type ProjectsSearch = {
  page?: number;
};

const PER_PAGE = 20;

export const Route = createFileRoute("/app/projects/")({
  validateSearch: (search: Record<string, unknown>): ProjectsSearch => {
    const page = Number(search.page);
    if (Number.isInteger(page) && page >= 1) {
      return { page };
    }
    return {};
  },
  component: ProjectsPage,
});

function ProjectsPage() {
  const page = Route.useSearch().page ?? 1;
  const list = useQuery(
    listProjectsOptions({ query: { page, per_page: PER_PAGE } }),
  );

  if (list.isPending) {
    return (
      <div className="flex flex-col gap-3">
        <Skeleton className="h-8 w-40" />
        <Skeleton className="h-24 w-full" />
      </div>
    );
  }

  if (list.isError) {
    return <p className="text-destructive text-sm">Could not load projects.</p>;
  }

  const data = list.data;
  const totalPages = Math.max(1, Math.ceil(data.total / data.per_page));

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between gap-3">
        <h1 className="font-heading text-xl font-medium">Projects</h1>
        <Button asChild>
          <Link to="/app/projects/new">New project</Link>
        </Button>
      </div>
      {data.items.length === 0 ? (
        <p className="text-muted-foreground text-sm">
          No projects yet. Create one to get started.
        </p>
      ) : (
        <ul className="flex flex-col gap-3">
          {data.items.map((project) => (
            <li key={project.id}>
              <Link
                to="/app/projects/$id"
                params={{ id: project.id }}
                className="block"
              >
                <Card>
                  <CardHeader>
                    <CardTitle>{project.name}</CardTitle>
                    {project.description ? (
                      <CardDescription>{project.description}</CardDescription>
                    ) : null}
                  </CardHeader>
                </Card>
              </Link>
            </li>
          ))}
        </ul>
      )}
      {totalPages > 1 ? (
        <div className="flex items-center justify-between gap-3">
          <Button variant="outline" asChild disabled={page <= 1}>
            <Link
              to="/app/projects"
              search={{ page: Math.max(1, page - 1) }}
              disabled={page <= 1}
            >
              Previous
            </Link>
          </Button>
          <span className="text-muted-foreground text-sm">
            Page {data.page} of {totalPages}
          </span>
          <Button variant="outline" asChild disabled={page >= totalPages}>
            <Link
              to="/app/projects"
              search={{ page: page + 1 }}
              disabled={page >= totalPages}
            >
              Next
            </Link>
          </Button>
        </div>
      ) : null}
    </div>
  );
}
