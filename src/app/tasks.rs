use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use servicez_application::error::AppError;
use servicez_http::{
    routes::tasks::TaskOperations,
    schemas::response::{CommentListResponse, CommentResponse, TaskPageResponse, TaskResponse},
};

use super::state::AppState;

impl TaskOperations for AppState {
    fn create_task(
        &self,
        caller_id: uuid::Uuid,
        subject: String,
        description: Option<String>,
        assignee_id: Option<uuid::Uuid>,
    ) -> impl std::future::Future<Output = Result<TaskResponse, AppError>> + Send {
        let svc = Arc::clone(&self.task_service);
        async move {
            svc.create_task(caller_id, subject, description, assignee_id)
                .await
                .map(TaskResponse::from)
        }
    }

    fn edit_task(
        &self,
        caller_id: uuid::Uuid,
        task_id: uuid::Uuid,
        subject: Option<String>,
        description: Option<Option<String>>,
        status: Option<String>,
    ) -> impl std::future::Future<Output = Result<TaskResponse, AppError>> + Send {
        let svc = Arc::clone(&self.task_service);
        async move {
            svc.edit_task(caller_id, task_id, subject, description, status)
                .await
                .map(TaskResponse::from)
        }
    }

    fn assign_task(
        &self,
        caller_id: uuid::Uuid,
        task_id: uuid::Uuid,
        assignee_id: uuid::Uuid,
    ) -> impl std::future::Future<Output = Result<TaskResponse, AppError>> + Send {
        let svc = Arc::clone(&self.task_service);
        async move {
            svc.assign_task(caller_id, task_id, assignee_id)
                .await
                .map(TaskResponse::from)
        }
    }

    fn list_tasks(
        &self,
        caller_id: Uuid,
        statuses: Vec<String>,
        limit: u32,
        cursor: Option<(DateTime<Utc>, Uuid)>,
    ) -> impl std::future::Future<Output = Result<TaskPageResponse, AppError>> + Send {
        let svc = Arc::clone(&self.task_service);
        async move {
            let page = svc.list_tasks(caller_id, statuses, limit, cursor).await?;
            let next_cursor = page
                .next_cursor
                .map(|c| servicez_http::cursor::encode(c.created_at, *c.id.as_uuid()));
            Ok(TaskPageResponse {
                items: page.items.into_iter().map(TaskResponse::from).collect(),
                next_cursor,
            })
        }
    }

    fn add_comment(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        body: String,
    ) -> impl std::future::Future<Output = Result<CommentResponse, AppError>> + Send {
        let svc = Arc::clone(&self.task_service);
        async move {
            svc.add_comment(caller_id, task_id, body)
                .await
                .map(CommentResponse::from)
        }
    }

    fn list_comments(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        limit: u32,
        cursor: Option<(DateTime<Utc>, Uuid)>,
    ) -> impl std::future::Future<Output = Result<CommentListResponse, AppError>> + Send {
        let svc = Arc::clone(&self.task_service);
        async move {
            let page = svc.list_comments(caller_id, task_id, limit, cursor).await?;
            let next_cursor = page
                .next_cursor
                .map(|c| servicez_http::cursor::encode(c.created_at, *c.id.as_uuid()));
            Ok(CommentListResponse {
                items: page.items.into_iter().map(CommentResponse::from).collect(),
                next_cursor,
            })
        }
    }

    fn edit_comment(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        comment_id: Uuid,
        body: String,
    ) -> impl std::future::Future<Output = Result<CommentResponse, AppError>> + Send {
        let svc = Arc::clone(&self.task_service);
        async move {
            svc.edit_comment(caller_id, task_id, comment_id, body)
                .await
                .map(CommentResponse::from)
        }
    }

    fn delete_comment(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        comment_id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send {
        let svc = Arc::clone(&self.task_service);
        async move { svc.delete_comment(caller_id, task_id, comment_id).await }
    }
}
