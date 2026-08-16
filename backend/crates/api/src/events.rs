use uuid::Uuid;

/// Broadcast on every task/comment/attachment/tag mutation so open board
/// sessions can refetch instead of polling. Best-effort: `send` errors when
/// there are no subscribers are ignored by every call site.
#[derive(Debug, Clone, Copy)]
pub struct ProjectEvent {
    pub project_id: Uuid,
}
