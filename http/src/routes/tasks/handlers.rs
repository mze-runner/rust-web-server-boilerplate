use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use servicez_application::error::AppError;

use crate::{
    error::app_error_to_response,
    extractors::{
        auth::AuthenticatedUser,
        validation::{ValidatedJson, ValidatedQuery},
    },
    middleware::trace::TraceId,
    schemas::requests::{
        AddCommentRequest, AssignTaskRequest, CreateTaskRequest, EditCommentRequest,
        EditTaskRequest, ListCommentsParams, ListTasksParams,
    },
};

use super::TaskOperations;

pub async fn list_tasks_handler<S: TaskOperations>(
    State(state): State<Arc<S>>,
    Extension(trace): Extension<TraceId>,
    AuthenticatedUser(subject): AuthenticatedUser,
    ValidatedQuery(params): ValidatedQuery<ListTasksParams>,
) -> Response {
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
        .list_tasks(subject.0, params.statuses, params.limit.0, cursor)
        .await
    {
        Ok(page) => (StatusCode::OK, Json(page)).into_response(),
        Err(e) => app_error_to_response(e, trace.as_str()),
    }
}

pub async fn create_task<S: TaskOperations>(
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

pub async fn assign_task_handler<S: TaskOperations>(
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

pub async fn edit_task<S: TaskOperations>(
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

pub async fn add_comment_handler<S: TaskOperations>(
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

pub async fn list_comments_handler<S: TaskOperations>(
    State(state): State<Arc<S>>,
    Extension(trace): Extension<TraceId>,
    AuthenticatedUser(subject): AuthenticatedUser,
    Path(task_id): Path<Uuid>,
    ValidatedQuery(params): ValidatedQuery<ListCommentsParams>,
) -> Response {
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
        .list_comments(caller_id, task_id, params.limit.0, cursor)
        .await
    {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => app_error_to_response(e, trace.as_str()),
    }
}

pub async fn edit_comment_handler<S: TaskOperations>(
    State(state): State<Arc<S>>,
    Extension(trace): Extension<TraceId>,
    AuthenticatedUser(subject): AuthenticatedUser,
    Path((task_id, comment_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(body): ValidatedJson<EditCommentRequest>,
) -> Response {
    let caller_id = subject.0;
    let comment_body = body.body.into_string();

    match state
        .edit_comment(caller_id, task_id, comment_id, comment_body)
        .await
    {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => app_error_to_response(e, trace.as_str()),
    }
}

pub async fn delete_comment_handler<S: TaskOperations>(
    State(state): State<Arc<S>>,
    Extension(trace): Extension<TraceId>,
    AuthenticatedUser(subject): AuthenticatedUser,
    Path((task_id, comment_id)): Path<(Uuid, Uuid)>,
) -> Response {
    let caller_id = subject.0;

    match state.delete_comment(caller_id, task_id, comment_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => app_error_to_response(e, trace.as_str()),
    }
}
