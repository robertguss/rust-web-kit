import { Link } from "@tanstack/react-router";

import { ThemeToggle } from "@/components/theme-toggle";
import { Button } from "@/components/ui/button";

export function PublicHeader() {
  return (
    <header className="flex items-center justify-between gap-4 border-b px-4 py-3">
      <Link to="/" className="font-heading text-base font-medium">
        rwk
      </Link>
      <nav className="flex items-center gap-2">
        <Button variant="ghost" size="sm" asChild>
          <Link to="/login">Log in</Link>
        </Button>
        <Button size="sm" asChild>
          <Link to="/register">Register</Link>
        </Button>
        <ThemeToggle />
      </nav>
    </header>
  );
}
