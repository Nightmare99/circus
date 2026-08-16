import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { getAccessToken } from "./api";

/**
 * Subscribes to /api/projects/{id}/ws and invalidates the board's task
 * queries whenever the server broadcasts a change, so edits from other
 * users/tabs show up without a manual refresh. Reconnects with a short
 * fixed backoff; the socket carries no data of its own, so a missed
 * message just means the next poll-worthy refetch happens a beat later.
 */
export function useProjectLiveUpdates(projectId: string | undefined) {
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!projectId) return;
    let socket: WebSocket | null = null;
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let stopped = false;

    const connect = () => {
      const token = getAccessToken();
      if (!token || stopped) return;
      const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
      socket = new WebSocket(
        `${protocol}//${window.location.host}/api/projects/${projectId}/ws?token=${encodeURIComponent(token)}`,
      );
      socket.onmessage = () => {
        queryClient.invalidateQueries({ queryKey: ["tasks", projectId] });
        queryClient.invalidateQueries({ queryKey: ["task"] });
      };
      socket.onclose = () => {
        if (!stopped) reconnectTimer = setTimeout(connect, 3000);
      };
      socket.onerror = () => socket?.close();
    };

    connect();

    return () => {
      stopped = true;
      if (reconnectTimer) clearTimeout(reconnectTimer);
      socket?.close();
    };
  }, [projectId, queryClient]);
}
