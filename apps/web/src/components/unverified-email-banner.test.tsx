import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { UnverifiedEmailBanner } from "./unverified-email-banner";

const DISMISS_KEY = "rwk:unverified-email-banner-dismissed";

function renderBanner(emailVerified: boolean) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <UnverifiedEmailBanner
        email="a@example.com"
        emailVerified={emailVerified}
      />
    </QueryClientProvider>,
  );
}

describe("UnverifiedEmailBanner", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    sessionStorage.removeItem(DISMISS_KEY);
  });

  it("renders when the address is unverified", () => {
    renderBanner(false);
    expect(screen.getByRole("status")).toHaveTextContent(
      /a@example.com is not verified/i,
    );
    expect(screen.getByRole("button", { name: "Resend" })).toBeInTheDocument();
  });

  it("hides when the address is verified", () => {
    renderBanner(true);
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Resend" }),
    ).not.toBeInTheDocument();
  });

  it("calls the resend-verification endpoint on click", async () => {
    const user = userEvent.setup();
    renderBanner(false);
    const fetchMock = vi.fn(async (_input: RequestInfo | URL) => {
      return new Response(null, { status: 204 });
    });
    vi.stubGlobal("fetch", fetchMock);

    await user.click(screen.getByRole("button", { name: "Resend" }));

    await vi.waitFor(() => {
      expect(fetchMock).toHaveBeenCalled();
    });
    const first = fetchMock.mock.calls[0]?.[0];
    const url = first instanceof Request ? first.url : String(first);
    expect(url).toContain("/auth/resend-verification");
  });
});
