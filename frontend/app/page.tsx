"use client";

import HeroSection from "./components/HeroSection";

// 主页界面
export default function Home() {
  return (
    <div className="flex flex-col w-full min-h-[200dvh] pt-[var(--header-h)]">
      <HeroSection />
      <div className="flex flex-row w-full">
        <div className="flex flex-col"></div>
      </div>
    </div>
  );
}
