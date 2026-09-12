import { Skeleton } from "@/components/ui/skeleton";

export function PendingScreen() {
  return (
    <div className="flex min-h-svh flex-col gap-4 p-6">
      <Skeleton className="h-8 w-48" />
      <Skeleton className="h-24 w-full max-w-xl" />
      <Skeleton className="h-24 w-full max-w-xl" />
    </div>
  );
}
