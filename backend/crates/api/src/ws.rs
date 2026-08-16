use crate::{auth::ProjectScope, state::AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

/// Live board updates: the client opens one socket per project (see
/// ProjectBoardPage) and refetches its task list whenever it receives a
/// message, rather than the server pushing full state over the wire.
pub async fn project_ws(
    State(state): State<AppState>,
    scope: ProjectScope,
    ws: WebSocketUpgrade,
) -> Response {
    let project_id = scope.project_id;
    ws.on_upgrade(move |socket| handle_socket(socket, state, project_id))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, project_id: Uuid) {
    let mut rx = state.events.subscribe();
    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(e) if e.project_id == project_id => {
                        if socket.send(Message::Text("tasks_changed".into())).await.is_err() {
                            break;
                        }
                    }
                    Ok(_) => continue,
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    None | Some(Err(_)) => break,
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(_)) => {} // ignore pings and any client messages
                }
            }
        }
    }
}
