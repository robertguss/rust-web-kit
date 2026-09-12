import { type APIRequestContext, expect, test } from "@playwright/test";

const MAILPIT = "http://localhost:8025/api/v1";

async function waitForVerifyLink(
  request: APIRequestContext,
  email: string,
): Promise<string> {
  const deadline = Date.now() + 20_000;
  while (Date.now() < deadline) {
    const list = await request.get(`${MAILPIT}/messages`);
    expect(list.ok()).toBeTruthy();
    const body = (await list.json()) as {
      messages?: Array<{ ID: string; To?: Array<{ Address?: string }> }>;
    };
    const match = (body.messages ?? []).find((message) =>
      (message.To ?? []).some(
        (to) => to.Address?.toLowerCase() === email.toLowerCase(),
      ),
    );
    if (match) {
      const full = await request.get(`${MAILPIT}/message/${match.ID}`);
      expect(full.ok()).toBeTruthy();
      const msg = (await full.json()) as { Text?: string; HTML?: string };
      const text = `${msg.Text ?? ""}\n${msg.HTML ?? ""}`;
      const found = text.match(
        /https?:\/\/[^\s"'<>]+\/verify-email\?token=[^\s"'<>]+/,
      );
      if (found) {
        return found[0];
      }
    }
    await new Promise((resolve) => setTimeout(resolve, 400));
  }
  throw new Error(`Mailpit had no verify link for ${email}`);
}

test("register, verify via Mailpit, login, project CRUD, logout", async ({
  page,
  request,
}) => {
  const email = `e2e-${Date.now()}@example.com`;
  const password = "password12";

  await page.goto("/register");
  await page.locator("#email").fill(email);
  await page.locator("#password").fill(password);
  await page.getByRole("button", { name: "Create account" }).click();
  await page.waitForURL("**/app/projects**");

  const verifyUrl = await waitForVerifyLink(request, email);
  await page.goto(verifyUrl);
  await expect(page.getByText("Email verified.")).toBeVisible();
  await page.getByRole("link", { name: "Continue" }).click();
  await page.waitForURL("**/app/projects**");

  await page.getByRole("button", { name: "Log out" }).click();
  await page.waitForURL("**/login**");

  await page.locator("#email").fill(email);
  await page.locator("#password").fill(password);
  await page.getByRole("button", { name: "Log in" }).click();
  await page.waitForURL("**/app/projects**");

  await page.getByRole("link", { name: "New project" }).click();
  await page.waitForURL("**/app/projects/new**");
  await page.locator("#name").fill("E2E project");
  await page.locator("#description").fill("created by playwright");
  await page.getByRole("button", { name: "Create project" }).click();
  await page.waitForURL(/\/app\/projects\/[^/]+$/);
  await expect(
    page.getByRole("heading", { name: "E2E project" }),
  ).toBeVisible();

  await page.locator("#name").fill("E2E project edited");
  await page.getByRole("button", { name: "Save changes" }).click();
  await expect(
    page.getByRole("heading", { name: "E2E project edited" }),
  ).toBeVisible();

  await page.getByRole("button", { name: "Delete" }).click();
  await page
    .getByRole("alertdialog")
    .getByRole("button", { name: "Delete" })
    .click();
  await page.waitForURL("**/app/projects");
  await expect(page.getByText("No projects yet.")).toBeVisible();

  await page.getByRole("button", { name: "Log out" }).click();
  await page.waitForURL("**/login**");
  await expect(page.getByRole("button", { name: "Log in" })).toBeVisible();
});
