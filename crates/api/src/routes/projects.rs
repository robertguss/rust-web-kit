//! `/api/projects` owner-scoped CRUD.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use garde::Validate;
use rwk_core::AppError;
use rwk_core::AppState;
use rwk_core::auth::RequireAuth;
use rwk_core::error::Problem;
use rwk_core::projects::service;
use rwk_core::projects::{Project, ProjectPage};
use uuid::Uuid;

use crate::dto::{CreateProject, PageQuery, UpdateProject};

/// List the current user's projects.
#[utoipa::path(
    get,
    path = "/",
    tag = "projects",
    params(PageQuery),
    responses(
        (status = 200, description = "Page of projects", body = ProjectPage),
        (status = 401, description = "Not authenticated", body = Problem),
        (status = 422, description = "Validation failed", body = Problem),
    )
)]
pub async fn list_projects(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Query(query): Query<PageQuery>,
) -> Result<Json<ProjectPage>, AppError> {
    query.validate()?;
    let page = service::list(state.db(), user.id, query.page, query.per_page).await?;
    Ok(Json(page))
}

/// Create a project owned by the current user.
#[utoipa::path(
    post,
    path = "/",
    tag = "projects",
    request_body = CreateProject,
    responses(
        (status = 201, description = "Created", body = Project),
        (status = 401, description = "Not authenticated", body = Problem),
        (status = 422, description = "Validation failed", body = Problem),
    )
)]
pub async fn create_project(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Json(body): Json<CreateProject>,
) -> Result<(StatusCode, Json<Project>), AppError> {
    body.validate()?;
    let project =
        service::create(state.db(), user.id, &body.name, body.description.as_deref()).await?;
    Ok((StatusCode::CREATED, Json(project)))
}

/// Fetch one owned project. Other users see 404.
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "projects",
    params(("id" = Uuid, Path, description = "Project id")),
    responses(
        (status = 200, description = "Project", body = Project),
        (status = 401, description = "Not authenticated", body = Problem),
        (status = 404, description = "Not found", body = Problem),
    )
)]
pub async fn get_project(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<Project>, AppError> {
    let project = service::get(state.db(), user.id, id).await?;
    Ok(Json(project))
}

/// Patch an owned project.
#[utoipa::path(
    patch,
    path = "/{id}",
    tag = "projects",
    params(("id" = Uuid, Path, description = "Project id")),
    request_body = UpdateProject,
    responses(
        (status = 200, description = "Updated", body = Project),
        (status = 401, description = "Not authenticated", body = Problem),
        (status = 404, description = "Not found", body = Problem),
        (status = 422, description = "Validation failed", body = Problem),
    )
)]
pub async fn update_project(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateProject>,
) -> Result<Json<Project>, AppError> {
    body.validate()?;
    let project = service::update(
        state.db(),
        user.id,
        id,
        body.name.as_deref(),
        body.description.as_deref(),
    )
    .await?;
    Ok(Json(project))
}

/// Delete an owned project.
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "projects",
    params(("id" = Uuid, Path, description = "Project id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Not authenticated", body = Problem),
        (status = 404, description = "Not found", body = Problem),
    )
)]
pub async fn delete_project(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    service::delete(state.db(), user.id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
