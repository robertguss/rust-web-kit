import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRootRoute,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { LoginForm } from "./login-form";

function renderLoginForm() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const rootRoute = createRootRoute({
    component: () => <LoginForm />,
  });
  const router = createRouter({
    routeTree: rootRoute,
    history: createMemoryHistory({ initialEntries: ["/"] }),
    context: { queryClient },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

describe("LoginForm", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("shows client-side validation messages", async () => {
    const user = userEvent.setup();
    renderLoginForm();

    await user.click(await screen.findByRole("button", { name: "Log in" }));

    expect(await screen.findByText("Enter a valid email")).toBeInTheDocument();
    expect(
      screen.getByText("Password must be at least 8 characters"),
    ).toBeInTheDocument();
  });

  it("maps server field errors onto inputs", async () => {
    const user = userEvent.setup();
    renderLoginForm();

    await user.type(await screen.findByLabelText("Email"), "taken@example.com");
    await user.type(screen.getByLabelText("Password"), "password12");

    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        return new Response(
          JSON.stringify({
            type: "about:blank",
            title: "Unprocessable Entity",
            status: 422,
            detail: "One or more fields failed validation.",
            errors: { email: ["already registered"] },
          }),
          {
            status: 422,
            headers: { "content-type": "application/problem+json" },
          },
        );
      }),
    );

    await user.click(screen.getByRole("button", { name: "Log in" }));

    expect(await screen.findByText("already registered")).toBeInTheDocument();
  });
});
