import { Button } from "@/components/ui/button";

const providers = [
  { id: "google", label: "Continue with Google" },
  { id: "github", label: "Continue with GitHub" },
] as const;

export function OauthButtons() {
  return (
    <div className="flex flex-col gap-2">
      {providers.map((provider) => (
        <Button key={provider.id} variant="outline" asChild>
          <a href={`/api/auth/oauth/${provider.id}`}>{provider.label}</a>
        </Button>
      ))}
    </div>
  );
}
