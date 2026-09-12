---
name: add-page
description: Add a TanStack file-based frontend route and page. Use when adding a screen, form page, or authed app route.
---

# Add a page

API hooks come from `just gen`. Do not invent fetch wrappers for documented endpoints.

## Files to touch

- `apps/web/src/routes/…tsx` — file-based route (`/app` is authed via `beforeLoad`)
- `apps/web/src/components/…` — form or view pieces
- `apps/web/src/lib/schemas.ts` — zod, matching garde limits
- `apps/web/src/components/app-nav.tsx` — link, if the page is primary nav
- `apps/web/src/routeTree.gen.ts` — written by the Vite plugin; do not edit

Hand-written API helpers stay in `apps/web/src/api/{client,problem,form-errors,query}.ts`. Never edit `src/api/generated/`.

## Route placement

| URL | File |
|---|---|
| `/` | `src/routes/index.tsx` |
| `/login` | `src/routes/login.tsx` |
| `/app/…` | `src/routes/app/….tsx` (guarded) |
| `/app/projects/$id` | `src/routes/app/projects/$id.tsx` |

Authed pages go under `src/routes/app/`. Return-to after login is `/app*` only (`src/lib/return-to.ts`).

## Form pattern

react-hook-form + zod resolver. On submit failure, `applyProblem(error, form.setError)` so RFC 9457 `errors.field` land on inputs. Toasts via sonner.

Import `listXOptions` / `createXMutation` from `@/api/generated/@tanstack/react-query.gen`. Invalidate those query keys after mutations.

## Checklist

- [ ] `just gen` already includes the endpoint
- [ ] Zod limits match the API DTO
- [ ] Pending and error UI (skeleton / message)
- [ ] Authed route lives under `/app`
- [ ] Vitest if the form has non-trivial validation or error mapping
- [ ] `pnpm --dir apps/web typecheck` and `lint` (or `just check`)
