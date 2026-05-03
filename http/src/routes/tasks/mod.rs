mod handlers;

use std::future::Future;
use std::sync::Arc;

use axum::{
    routing::{get, patch, put},
    Router,
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use servicez_application::error::AppError;

use crate::schemas::response::{
    CommentListResponse, CommentResponse, TaskPageResponse, TaskResponse,
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

    fn edit_comment(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        comment_id: Uuid,
        body: String,
    ) -> impl Future<Output = Result<CommentResponse, AppError>> + Send;

    fn delete_comment(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        comment_id: Uuid,
    ) -> impl Future<Output = Result<(), AppError>> + Send;
}

pub fn router<S>() -> Router<Arc<S>>
where
    S: TaskOperations,
{
    Router::new()
        .route(
            "/api/v1/tasks",
            get(handlers::list_tasks_handler::<S>).post(handlers::create_task::<S>),
        )
        .route("/api/v1/tasks/{id}", patch(handlers::edit_task::<S>))
        .route(
            "/api/v1/tasks/{id}/assignee",
            put(handlers::assign_task_handler::<S>),
        )
        .route(
            "/api/v1/tasks/{id}/comments",
            get(handlers::list_comments_handler::<S>).post(handlers::add_comment_handler::<S>),
        )
        .route(
            "/api/v1/tasks/{id}/comments/{comment_id}",
            put(handlers::edit_comment_handler::<S>).delete(handlers::delete_comment_handler::<S>),
        )
}
