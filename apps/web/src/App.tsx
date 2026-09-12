import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export default function App() {
  return (
    <main className="flex min-h-svh items-center justify-center p-6">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>rwk</CardTitle>
          <CardDescription>
            Rust (Axum) API + React (Vite) starter.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          <p className="text-muted-foreground text-sm">
            API health lives at <code>/api/health</code>.
          </p>
          <Button type="button">Get started</Button>
        </CardContent>
      </Card>
    </main>
  );
}
