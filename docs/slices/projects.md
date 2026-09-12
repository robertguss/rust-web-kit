# Slice: projects

Copy this slice when adding an owner-scoped CRUD resource.

## Shape

- Table `projects`: `id` uuid v7 PK, `owner_id` → `users`, `name`, `description`, `created_at`, `updated_at`. Index on `owner_id`.
- All queries filter `owner_id`. A row that exists for someone else is indistinguishable from missing: **404**.
- List is newest-first (`created_at DESC, id DESC`). Pagination: `page` default 1, `per_page` default 20 (garde 1–100).
- PATCH uses SQL `COALESCE` so omitted fields stay put (cannot null-clear `description`).

## Files

| Layer | Path |
|---|---|
| Migration | `migrations/1_init.sql` (`projects` + `projects_owner_id_idx`) |
| Model | `crates/core/src/projects/model.rs` — `Project`, `ProjectPage` |
| Repo | `crates/core/src/projects/repo.rs` — `create`, `find_owned`, `count_owned`, `list_owned`, `update_owned`, `delete_owned` |
| Service | `crates/core/src/projects/service.rs` — maps `None` / `false` → `AppError::NotFound` |
| Module | `crates/core/src/projects/mod.rs` (re-export model types) |
| DTOs | `crates/api/src/dto.rs` — `CreateProject`, `UpdateProject`, `PageQuery` |
| HTTP | `crates/api/src/routes/projects.rs` |
| Router | `crates/api/src/router.rs` — `nest("/projects", …)`, `ApiDoc` schemas/tag |
| Tests | `crates/api/tests/projects.rs` |
| UI list | `apps/web/src/routes/app/projects/index.tsx` |
| UI new | `apps/web/src/routes/app/projects/new.tsx` |
| UI edit | `apps/web/src/routes/app/projects/$id.tsx` |
| Form | `apps/web/src/components/projects/project-form.tsx` |
| Zod | `apps/web/src/lib/schemas.ts` — `projectSchema` |

`crates/core/src/lib.rs` already `pub mod projects`.

## HTTP (`/api` prefix on the wire)

| Method | Path | Auth | Status |
|---|---|---|---|
| GET | `/projects?page&per_page` | `RequireAuth` | 200 `ProjectPage` |
| POST | `/projects` | `RequireAuth` | 201 `Project` |
| GET | `/projects/{id}` | `RequireAuth` | 200 / 404 |
| PATCH | `/projects/{id}` | `RequireAuth` | 200 / 404 |
| DELETE | `/projects/{id}` | `RequireAuth` | 204 / 404 |

utoipa paths are **unprefixed** (`path = "/"`, `path = "/{id}"`) because the OpenAPI router is nested at `/projects` and the spec server URL is `/api`.

Handlers: `body.validate()?` then `projects::service::…`. Extract `RequireAuth(user)` and pass `user.id` as `owner_id`.

## Client / UI

1. `just gen` → `listProjectsOptions`, `createProjectMutation`, `getProjectOptions`, `updateProjectMutation`, `deleteProjectMutation` from `@/api/generated/@tanstack/react-query.gen`.
2. `/app` `beforeLoad` already redirects anonymous users to `/login`. Child routes inherit that.
3. Mutations `onSuccess` invalidate the list query key (see create/edit/delete components).
4. Server field errors: `applyProblem(error, form.setError)` from `@/api/form-errors`.

## Tests to copy

From `crates/api/tests/projects.rs`:

- unauthenticated list/create → 401 problem+json
- full CRUD
- other user get/patch/delete → 404; their list empty
- pagination (`per_page=2` over 3 rows)
- validation 422 (`errors.name`, `page=0`, empty PATCH name)

Register two users through the cookie client in `crates/api/tests/common`.
