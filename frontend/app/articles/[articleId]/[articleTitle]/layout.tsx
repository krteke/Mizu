import React from "react";

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex flex-row w-3/4 min-h-dvh pt-[var(--header-h)]">
      <div className="flex flex-col flex-1"></div>
      <div className="flex flex-col flex-3">{children}</div>
      <div className="flex flex-col flex-1"></div>
    </div>
  );
}
