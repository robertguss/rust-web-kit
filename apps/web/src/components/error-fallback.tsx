import { Link } from "@tanstack/react-router";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

function errorMessage(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message;
  }
  return "An unexpected error occurred.";
}

export function ErrorFallback({ error }: { error: unknown }) {
  return (
    <main className="flex min-h-svh items-center justify-center p-6">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Something went wrong</CardTitle>
          <CardDescription>{errorMessage(error)}</CardDescription>
        </CardHeader>
        <CardContent>
          <Button asChild>
            <Link to="/">Back home</Link>
          </Button>
        </CardContent>
      </Card>
    </main>
  );
}

export function NotFoundPage() {
  return (
    <main className="flex min-h-svh items-center justify-center p-6">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Not found</CardTitle>
          <CardDescription>That page does not exist.</CardDescription>
        </CardHeader>
        <CardContent>
          <Button asChild>
            <Link to="/">Back home</Link>
          </Button>
        </CardContent>
      </Card>
    </main>
  );
}
