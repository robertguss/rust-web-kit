import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { toast } from "sonner";

import { logoutMutation } from "@/api/generated/@tanstack/react-query.gen";
import { meQuery, meQueryKey } from "@/api/query";
import { ThemeToggle } from "@/components/theme-toggle";
import { Button } from "@/components/ui/button";

export function AppNav() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const me = useQuery(meQuery());
  const logout = useMutation({
    ...logoutMutation(),
    onSuccess: async () => {
      queryClient.removeQueries({ queryKey: meQueryKey() });
      queryClient.clear();
      toast.success("Signed out");
      await navigate({ to: "/login" });
    },
  });

  return (
    <header className="flex items-center justify-between gap-4 border-b px-4 py-3">
      <div className="flex items-center gap-4">
        <Link to="/app/projects" className="font-heading text-base font-medium">
          rwk
        </Link>
        <nav className="flex items-center gap-2">
          <Button variant="ghost" size="sm" asChild>
            <Link to="/app/projects">Projects</Link>
          </Button>
        </nav>
      </div>
      <div className="flex items-center gap-2">
        {me.data ? (
          <span className="text-muted-foreground hidden text-sm sm:inline">
            {me.data.email}
          </span>
        ) : null}
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={logout.isPending}
          onClick={() => logout.mutate({})}
        >
          Log out
        </Button>
        <ThemeToggle />
      </div>
    </header>
  );
}
