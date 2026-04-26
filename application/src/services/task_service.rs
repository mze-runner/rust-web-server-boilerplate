use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use std::str::FromStr;

use servicez_domain::{
    error::DomainError,
    ports::{TaskCommentRepository, TaskRepository, UnitOfWork, UnitOfWorkFactory, UserRepository},
    task::{EditTaskCommand, ListTasksQuery, Task, TaskCursor, TaskId, TaskPage, TaskStatus},
    task_comment::{CommentCursor, CommentPage, ListCommentsQuery, TaskComment, TaskCommentId},
    user::UserId,
};

use crate::error::AppError;

pub struct TaskService<R, F>
where
    R: TaskRepository,
    F: UnitOfWorkFactory,
{
    repo: Arc<R>,
    uow_factory: Arc<F>,
}

impl<R, F> TaskService<R, F>
where
    R: TaskRepository,
    F: UnitOfWorkFactory,
    <F::Uow as UnitOfWork>::Users: UserRepository,
    <F::Uow as UnitOfWork>::Tasks: TaskRepository,
    <F::Uow as UnitOfWork>::Comments: TaskCommentRepository,
{
    pub fn new(repo: Arc<R>, uow_factory: Arc<F>) -> Self {
        Self { repo, uow_factory }
    }

    pub async fn create_task(
        &self,
        caller_id: Uuid,
        subject: String,
        description: Option<String>,
        assignee_id: Option<Uuid>,
    ) -> Result<Task, AppError> {
        let caller_id = UserId::from_uuid(caller_id);
        let mut uow = self.uow_factory.begin().await.map_err(AppError::Domain)?;

        let assignee_id = match assignee_id {
            Some(id) => {
                let uid = UserId::from_uuid(id);
                uow.users()
                    .find_by_id(&uid)
                    .await
                    .map_err(AppError::Domain)?
                    .ok_or_else(|| {
                        AppError::UnprocessableEntity(
                            "assignee_id does not reference a known user".into(),
                        )
                    })?;
                uid
            }
            None => caller_id.clone(),
        };

        let now = chrono::Utc::now();
        let task = Task::create(subject, description, assignee_id, caller_id, now)
            .map_err(AppError::Domain)?;

        uow.tasks().create(&task).await.map_err(AppError::Domain)?;

        uow.commit().await.map_err(AppError::Domain)?;

        Ok(task)
    }

    pub async fn assign_task(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        assignee_id: Uuid,
    ) -> Result<Task, AppError> {
        let caller_id = UserId::from_uuid(caller_id);
        let task_id = TaskId::from_uuid(task_id);
        let new_assignee_id = UserId::from_uuid(assignee_id);

        let mut uow = self.uow_factory.begin().await.map_err(AppError::Domain)?;

        let mut task = uow
            .tasks()
            .find_by_id(&task_id)
            .await
            .map_err(AppError::Domain)?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    resource_type: "Task".into(),
                    identifier: task_id.as_uuid().to_string(),
                })
            })?;

        let is_visible = task.created_by() == &caller_id || task.assignee_id() == &caller_id;
        if !is_visible {
            return Err(AppError::Domain(DomainError::NotFound {
                resource_type: "Task".into(),
                identifier: task_id.as_uuid().to_string(),
            }));
        }

        uow.users()
            .find_by_id(&new_assignee_id)
            .await
            .map_err(AppError::Domain)?
            .ok_or_else(|| {
                AppError::UnprocessableEntity("assignee_id does not reference a known user".into())
            })?;

        task.assign(new_assignee_id, &caller_id, chrono::Utc::now())
            .map_err(|e| match e {
                DomainError::ImmutableResource => AppError::UnprocessableEntity(
                    "cannot reassign a task in Done or Cancelled status".into(),
                ),
                other => AppError::Domain(other),
            })?;

        uow.tasks().update(&task).await.map_err(AppError::Domain)?;
        uow.commit().await.map_err(AppError::Domain)?;

        Ok(task)
    }

    pub async fn list_tasks(
        &self,
        caller_id: Uuid,
        statuses: Vec<String>,
        limit: u32,
        cursor: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<TaskPage, AppError> {
        let caller_id = UserId::from_uuid(caller_id);
        let statuses = statuses
            .into_iter()
            .map(|s| TaskStatus::from_str(&s))
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::Domain)?;
        let cursor = cursor.map(|(created_at, id)| TaskCursor {
            created_at,
            id: TaskId::from_uuid(id),
        });
        let query = ListTasksQuery {
            caller_id,
            statuses,
            limit,
            cursor,
        };
        self.repo
            .list_for_user(&query)
            .await
            .map_err(AppError::Domain)
    }

    pub async fn add_comment(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        body: String,
    ) -> Result<TaskComment, AppError> {
        let caller_id = UserId::from_uuid(caller_id);
        let task_id = TaskId::from_uuid(task_id);

        let mut uow = self.uow_factory.begin().await.map_err(AppError::Domain)?;

        let task = uow
            .tasks()
            .find_by_id(&task_id)
            .await
            .map_err(AppError::Domain)?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    resource_type: "Task".into(),
                    identifier: task_id.as_uuid().to_string(),
                })
            })?;

        let can_comment = task.created_by() == &caller_id || task.assignee_id() == &caller_id;
        if !can_comment {
            return Err(AppError::Domain(DomainError::Forbidden {
                reason: "only the task creator or assignee may comment on this task".into(),
            }));
        }

        let now = chrono::Utc::now();
        let comment =
            TaskComment::create(task_id, caller_id, body, now).map_err(AppError::Domain)?;

        uow.comments()
            .create(&comment)
            .await
            .map_err(AppError::Domain)?;
        uow.commit().await.map_err(AppError::Domain)?;

        Ok(comment)
    }

    pub async fn list_comments(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        limit: u32,
        cursor: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<CommentPage, AppError> {
        let caller_id = UserId::from_uuid(caller_id);
        let task_id = TaskId::from_uuid(task_id);
        let cursor = cursor.map(|(created_at, id)| CommentCursor {
            created_at,
            id: TaskCommentId::from_uuid(id),
        });

        let mut uow = self.uow_factory.begin().await.map_err(AppError::Domain)?;

        let task = uow
            .tasks()
            .find_by_id(&task_id)
            .await
            .map_err(AppError::Domain)?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    resource_type: "Task".into(),
                    identifier: task_id.as_uuid().to_string(),
                })
            })?;

        let can_view = task.created_by() == &caller_id || task.assignee_id() == &caller_id;
        if !can_view {
            return Err(AppError::Domain(DomainError::Forbidden {
                reason: "only the task creator or assignee may view comments on this task".into(),
            }));
        }

        let query = ListCommentsQuery {
            task_id,
            limit,
            cursor,
        };
        uow.comments()
            .list_for_task(&query)
            .await
            .map_err(AppError::Domain)
    }

    pub async fn edit_comment(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        comment_id: Uuid,
        body: String,
    ) -> Result<TaskComment, AppError> {
        let caller_id = UserId::from_uuid(caller_id);
        let task_id = TaskId::from_uuid(task_id);
        let comment_id = TaskCommentId::from_uuid(comment_id);

        let mut uow = self.uow_factory.begin().await.map_err(AppError::Domain)?;

        let task = uow
            .tasks()
            .find_by_id(&task_id)
            .await
            .map_err(AppError::Domain)?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    resource_type: "Task".into(),
                    identifier: task_id.as_uuid().to_string(),
                })
            })?;

        let is_visible = task.created_by() == &caller_id || task.assignee_id() == &caller_id;
        if !is_visible {
            return Err(AppError::Domain(DomainError::NotFound {
                resource_type: "Task".into(),
                identifier: task_id.as_uuid().to_string(),
            }));
        }

        let mut comment = uow
            .comments()
            .find_by_id(&comment_id)
            .await
            .map_err(AppError::Domain)?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    resource_type: "TaskComment".into(),
                    identifier: comment_id.as_uuid().to_string(),
                })
            })?;

        if comment.task_id() != &task_id {
            return Err(AppError::Domain(DomainError::NotFound {
                resource_type: "TaskComment".into(),
                identifier: comment_id.as_uuid().to_string(),
            }));
        }

        if comment.author_id() != &caller_id {
            return Err(AppError::Domain(DomainError::Forbidden {
                reason: "only the comment author may edit this comment".into(),
            }));
        }

        let now = chrono::Utc::now();
        comment
            .edit(body, caller_id, now)
            .map_err(AppError::Domain)?;

        uow.comments()
            .update(&comment)
            .await
            .map_err(AppError::Domain)?;
        uow.commit().await.map_err(AppError::Domain)?;

        Ok(comment)
    }

    pub async fn edit_task(
        &self,
        caller_id: Uuid,
        task_id: Uuid,
        subject: Option<String>,
        description: Option<Option<String>>,
        status: Option<String>,
    ) -> Result<Task, AppError> {
        let caller_id = UserId::from_uuid(caller_id);
        let task_id = TaskId::from_uuid(task_id);

        let status = status
            .map(|s| TaskStatus::from_str(&s))
            .transpose()
            .map_err(AppError::Domain)?;

        let mut uow = self.uow_factory.begin().await.map_err(AppError::Domain)?;

        let mut task = uow
            .tasks()
            .find_by_id(&task_id)
            .await
            .map_err(AppError::Domain)?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    resource_type: "Task".into(),
                    identifier: task_id.as_uuid().to_string(),
                })
            })?;

        let cmd = EditTaskCommand {
            subject,
            description,
            status,
        };

        task.modify(&cmd, &caller_id, chrono::Utc::now())
            .map_err(AppError::Domain)?;

        uow.tasks().update(&task).await.map_err(AppError::Domain)?;

        uow.commit().await.map_err(AppError::Domain)?;

        Ok(task)
    }
}
