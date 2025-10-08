"use client";

import { useEffect, useState } from "react";

// 一个显示页面滚动进度的组件
export default function ScrollProgress() {
  const [scrollPercent, setScrollPercent] = useState(0);

  // 监听滚动事件，计算滚动百分比
  useEffect(() => {
    const scrollHandler = () => {
      const scrollTop = document.documentElement.scrollTop || 0;
      const scrollHeight = document.documentElement.scrollHeight || 0;
      const clientHeight = document.documentElement.clientHeight || 0;

      if (scrollHeight > clientHeight) {
        const percent = (scrollTop / (scrollHeight - clientHeight)) * 100;
        setScrollPercent(percent);
      } else {
        setScrollPercent(0);
      }
    };

    window.addEventListener("scroll", scrollHandler);

    return () => {
      window.removeEventListener("scroll", scrollHandler);
    };
  }, []);

  return (
    <div className="fixed w-full h-1.5 top-0 z-[9999]">
      <div
        style={{ width: `${scrollPercent}%` }}
        className="h-full bg-[#008C8C] transition-[width] duration-100 ease-out"
      ></div>
    </div>
  );
}
