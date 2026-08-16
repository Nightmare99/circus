import clsx from "clsx";
import type { ButtonHTMLAttributes, InputHTMLAttributes, ReactNode, SelectHTMLAttributes } from "react";
import type { Priority, TaskStatus } from "../lib/types";

export function Button({
  variant = "primary",
  className,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: "primary" | "secondary" | "ghost" | "danger" }) {
  return (
    <button
      className={clsx(
        "inline-flex items-center justify-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50",
        variant === "primary" && "bg-accent text-bg hover:bg-accent-strong",
        variant === "secondary" &&
          "border border-border-strong bg-surface-2 text-text hover:bg-surface-3",
        variant === "ghost" && "text-text-dim hover:bg-surface-2 hover:text-text",
        variant === "danger" && "bg-status-blocked/90 text-white hover:bg-status-blocked",
        className,
      )}
      {...props}
    />
  );
}

export function Input({ className, ...props }: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      className={clsx(
        "w-full rounded-md border border-border-strong bg-surface-2 px-3 py-1.5 text-sm text-text placeholder:text-text-faint focus:border-accent",
        className,
      )}
      {...props}
    />
  );
}

export function Textarea({
  className,
  ...props
}: React.TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return (
    <textarea
      className={clsx(
        "w-full resize-y rounded-md border border-border-strong bg-surface-2 px-3 py-1.5 text-sm text-text placeholder:text-text-faint focus:border-accent",
        className,
      )}
      {...props}
    />
  );
}

export function Select({ className, ...props }: SelectHTMLAttributes<HTMLSelectElement>) {
  return (
    <select
      className={clsx(
        "rounded-md border border-border-strong bg-surface-2 px-2 py-1.5 text-sm text-text focus:border-accent",
        className,
      )}
      {...props}
    />
  );
}

export function Label({ children }: { children: ReactNode }) {
  return (
    <label className="mb-1 block text-xs font-medium tracking-wide text-text-dim uppercase">
      {children}
    </label>
  );
}

export function Avatar({ name, size = 24 }: { name: string; size?: number }) {
  const initials = name
    .split(/\s+/)
    .slice(0, 2)
    .map((s) => s[0]?.toUpperCase())
    .join("");
  const hue = Math.abs(hashCode(name)) % 360;
  return (
    <span
      className="inline-flex shrink-0 items-center justify-center rounded-full font-semibold text-bg ring-1 ring-black/20"
      style={{
        width: size,
        height: size,
        fontSize: size * 0.4,
        background: `hsl(${hue} 65% 65%)`,
      }}
      title={name}
    >
      {initials || "?"}
    </span>
  );
}

function hashCode(s: string) {
  let h = 0;
  for (let i = 0; i < s.length; i++) h = (h << 5) - h + s.charCodeAt(i);
  return h;
}

export function TagChip({ name, color, onRemove }: { name: string; color: string; onRemove?: () => void }) {
  return (
    <span
      className="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium"
      style={{ backgroundColor: `${color}26`, color }}
    >
      {name}
      {onRemove && (
        <button onClick={onRemove} className="opacity-70 hover:opacity-100" aria-label={`Remove ${name}`}>
          ×
        </button>
      )}
    </span>
  );
}

export function StatusDot({ status }: { status: TaskStatus }) {
  return <span className={clsx("inline-block h-2 w-2 rounded-full", statusDotClass[status])} />;
}

const statusDotClass: Record<TaskStatus, string> = {
  pending: "bg-status-pending",
  in_progress: "bg-status-in_progress",
  blocked: "bg-status-blocked",
  completed: "bg-status-completed",
};

export function PriorityDot({ priority }: { priority: Priority }) {
  return (
    <span
      className={clsx("inline-block h-1.5 w-1.5 rounded-full", priorityDotClass[priority])}
      title={`${priority} priority`}
    />
  );
}

const priorityDotClass: Record<Priority, string> = {
  low: "bg-priority-low",
  medium: "bg-priority-medium",
  high: "bg-priority-high",
  urgent: "bg-priority-urgent",
};

export function Spinner({ className }: { className?: string }) {
  return (
    <span
      className={clsx(
        "inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent",
        className,
      )}
    />
  );
}

export function EmptyState({ title, hint }: { title: string; hint?: string }) {
  return (
    <div className="flex flex-col items-center justify-center gap-1 py-16 text-center">
      <p className="text-sm font-medium text-text-dim">{title}</p>
      {hint && <p className="text-xs text-text-faint">{hint}</p>}
    </div>
  );
}

export function ErrorBanner({ message }: { message: string }) {
  return (
    <div className="rounded-md border border-status-blocked/40 bg-status-blocked/10 px-3 py-2 text-sm text-status-blocked">
      {message}
    </div>
  );
}

export function Modal({
  title,
  onClose,
  children,
}: {
  title: string;
  onClose: () => void;
  children: ReactNode;
}) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md rounded-lg border border-border bg-surface p-5 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-sm font-semibold text-text">{title}</h2>
          <button
            onClick={onClose}
            className="text-text-faint hover:text-text"
            aria-label="Close"
          >
            ×
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}

export function Drawer({
  onClose,
  children,
}: {
  onClose: () => void;
  children: ReactNode;
}) {
  return (
    <div className="fixed inset-0 z-40 flex justify-end bg-black/50" onClick={onClose}>
      <div
        className="h-full w-full max-w-xl overflow-y-auto border-l border-border bg-surface shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        {children}
      </div>
    </div>
  );
}
