"use client";

import Toolbar from "./components/Toolbar";

// 主页界面
export default function Home() {
  return (
    <main className="flex flex-col w-full min-h-[200dvh] pt-[var(--header-h)]">
      <div className="h-[calc(100dvh-var(--header-h))] w-full flex bg-amber-50">
        <div></div>
      </div>
      <div className="flex flex-col w-full min-h-dvh bg-black"></div>
      <Toolbar />
    </main>
  );
}
