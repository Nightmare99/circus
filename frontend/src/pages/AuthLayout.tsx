import type { ReactNode } from "react";

export default function AuthLayout({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div className="flex min-h-screen items-center justify-center bg-bg px-4">
      <div className="w-full max-w-sm">
        <div className="mb-8 text-center">
          <span className="font-mono text-lg font-bold tracking-tight text-accent">circus</span>
          <div className="marquee-underline mx-auto mt-2 w-16" />
        </div>
        <div className="rounded-lg border border-border bg-surface p-6">
          <h1 className="mb-5 text-base font-semibold text-text">{title}</h1>
          {children}
        </div>
      </div>
    </div>
  );
}
