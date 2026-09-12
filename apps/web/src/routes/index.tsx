import { useQuery } from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";

import { meQuery } from "@/api/query";
import { PublicHeader } from "@/components/public-header";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export const Route = createFileRoute("/")({
  component: LandingPage,
});

function LandingPage() {
  const me = useQuery(meQuery());
  const signedIn = Boolean(me.data);

  return (
    <div className="flex min-h-svh flex-col">
      <PublicHeader />
      <main className="flex flex-1 items-center justify-center p-6">
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle>rwk</CardTitle>
            <CardDescription>
              Rust (Axum) API + React (Vite) starter.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            <p className="text-muted-foreground text-sm">
              Projects, sessions, and email-backed auth — one binary in
              production.
            </p>
            {signedIn ? (
              <Button asChild>
                <Link to="/app/projects">Open app</Link>
              </Button>
            ) : (
              <div className="flex flex-col gap-2">
                <Button asChild>
                  <Link to="/register">Get started</Link>
                </Button>
                <Button variant="outline" asChild>
                  <Link to="/login">Log in</Link>
                </Button>
              </div>
            )}
          </CardContent>
        </Card>
      </main>
    </div>
  );
}
