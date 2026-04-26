use std::future::Future;
use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use servicez_application::error::AppError;

use crate::{
    error::app_error_to_response,
    extractors::{auth::AuthenticatedUser, validation::ValidatedJson},
    middleware::trace::TraceId,
    schemas::{
        requests::{AddCommentRequest, AssignTaskRequest, CreateTaskRequest, EditTaskRequest},
        response::{CommentListResponse, CommentResponse, TaskPageResponse, TaskResponse},
    },
};

/// Port contract for task operations — implemented by `AppState` in the composition root.
pub trait TaskOperations: Clone + Send + Sync + 'static {
    fn create_task(
        &self,
        caller_id: Uuid,
        subject: String,
        description: Option<String>,
        assignee_id: Option<Uuid>,
    ) -> impl Future<Output = Result<TaskResponse, AppError>> + Send;

    fn edit_task(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        subject: Option<String>,
        description: Option<Option<String>>,
        status: Option<String>,
    ) -> impl Future<Output = Result<TaskResponse, AppError>> + Send;

    fn assign_task(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        assignee_id: Uuid,
    ) -> impl Future<Output = Result<TaskResponse, AppError>> + Send;

    fn list_tasks(
        &self,
        caller_id: Uuid,
        statuses: Vec<String>,
        limit: u32,
        cursor: Option<(DateTime<Utc>, Uuid)>,
    ) -> impl Future<Output = Result<TaskPageResponse, AppError>> + Send;

    fn add_comment(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        body: String,
    ) -> impl Future<Output = Result<CommentResponse, AppError>> + Send;

    fn list_comments(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        limit: u32,
        cursor: Option<(DateTime<Utc>, Uuid)>,
    ) -> impl Future<Output = Result<CommentListResponse, AppError>> + Send;
}

pub fn router<S>() -> Router<Arc<S>>
where
    S: TaskOperations,
{
    Router::new()
        .route("/", get(list_tasks_handler::<S>).post(create_task::<S>))
        .route("/{id}", patch(edit_task::<S>))
        .route("/{id}/assignee", put(assign_task_handler::<S>))
        .route(
            "/{id}/comments",
            get(list_comments_handler::<S>).post(add_comment_handler::<S>),
        )
}

#[derive(Deserialize)]
struct ListTasksParams {
    #[serde(default, rename = "status")]
    statuses: Vec<String>,
    #[serde(default = "default_limit")]
    limit: u32,
    cursor: Option<String>,
}

fn default_limit() -> u32 {
    20
}

async fn list_tasks_handler<S: TaskOperations>(
    State(state): State<Arc<S>>,
    Extension(trace): Extension<TraceId>,
    AuthenticatedUser(subject): AuthenticatedUser,
    Query(params): Query<ListTasksParams>,
) -> Response {
    if params.limit > 100 {
        return app_error_to_response(
            AppError::BadRequest("limit must not exceed 100".into()),
            trace.as_str(),
        );
    }

    let cursor = match params.cursor {
        None => None,
        Some(ref s) => match crate::cursor::decode(s) {
            Some(c) => Some(c),
            None => {
                return app_error_to_response(
                    AppError::BadRequest("invalid cursor".into()),
                    trace.as_str(),
                )
            }
        },
    };

    match state
        .list_tasks(subject.0, params.statuses, params.limit, cursor)
        .await
    {
        Ok(page) => (StatusCode::OK, Json(page)).into_response(),
        Err(e) => app_error_to_response(e, trace.as_str()),
    }
}

async fn create_task<S: TaskOperations>(
    State(state): State<Arc<S>>,
    Extension(trace): Extension<TraceId>,
    AuthenticatedUser(subject): AuthenticatedUser,
    ValidatedJson(body): ValidatedJson<CreateTaskRequest>,
) -> Response {
    let caller_id = subject.0;
    let subject_str = body.subject.into_string();
    let description = body.description.as_str().map(str::to_owned);

    match state
        .create_task(caller_id, subject_str, description, body.assignee_id)
        .await
    {
        Ok(response) => (StatusCode::CREATED, Json(response)).into_response(),
        Err(e) => app_error_to_response(e, trace.as_str()),
    }
}

async fn assign_task_handler<S: TaskOperations>(
    State(state): State<Arc<S>>,
    Extension(trace): Extension<TraceId>,
    AuthenticatedUser(subject): AuthenticatedUser,
    Path(task_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<AssignTaskRequest>,
) -> Response {
    let caller_id = subject.0;
    match state
        .assign_task(caller_id, task_id, body.assignee_id)
        .await
    {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => app_error_to_response(e, trace.as_str()),
    }
}

async fn edit_task<S: TaskOperations>(
    State(state): State<Arc<S>>,
    Extension(trace): Extension<TraceId>,
    AuthenticatedUser(subject): AuthenticatedUser,
    Path(task_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<EditTaskRequest>,
) -> Response {
    if body.is_empty() {
        return app_error_to_response(
            AppError::BadRequest("at least one field must be present".into()),
            trace.as_str(),
        );
    }

    let caller_id = subject.0;
    let req_subject = body.subject.and_then(|f| f.0);
    let description = body.description.map(|f| f.0);

    match state
        .edit_task(caller_id, task_id, req_subject, description, body.status)
        .await
    {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => app_error_to_response(e, trace.as_str()),
    }
}

async fn add_comment_handler<S: TaskOperations>(
    State(state): State<Arc<S>>,
    Extension(trace): Extension<TraceId>,
    AuthenticatedUser(subject): AuthenticatedUser,
    Path(task_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<AddCommentRequest>,
) -> Response {
    let caller_id = subject.0;
    let comment_body = body.body.into_string();

    match state.add_comment(caller_id, task_id, comment_body).await {
        Ok(response) => (StatusCode::CREATED, Json(response)).into_response(),
        Err(e) => app_error_to_response(e, trace.as_str()),
    }
}

#[derive(Deserialize)]
struct ListCommentsParams {
    #[serde(default = "default_limit")]
    limit: u32,
    cursor: Option<String>,
}

async fn list_comments_handler<S: TaskOperations>(
    State(state): State<Arc<S>>,
    Extension(trace): Extension<TraceId>,
    AuthenticatedUser(subject): AuthenticatedUser,
    Path(task_id): Path<Uuid>,
    Query(params): Query<ListCommentsParams>,
) -> Response {
    if params.limit > 100 {
        return app_error_to_response(
            AppError::BadRequest("limit must not exceed 100".into()),
            trace.as_str(),
        );
    }

    let cursor = match params.cursor {
        None => None,
        Some(ref s) => match crate::cursor::decode(s) {
            Some(c) => Some(c),
            None => {
                return app_error_to_response(
                    AppError::BadRequest("invalid cursor".into()),
                    trace.as_str(),
                )
            }
        },
    };

    let caller_id = subject.0;
    match state
        .list_comments(caller_id, task_id, params.limit, cursor)
        .await
    {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => app_error_to_response(e, trace.as_str()),
    }
}
