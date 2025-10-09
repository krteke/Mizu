"use client";

import { useEffect, useState } from "react";
import SearchBar from "./SearchBar";
import ScrollProgress from "./ScrollProgress";
import NavigateBar from "./NavigateBar";

export default function Header() {
  // 定义滚动方向的类型
  type ScrollDirection = "up" | "down" | null;

  // 使用状态来跟踪滚动方向和上次的滚动位置
  const [scrollDir, setScrollDir] = useState<ScrollDirection>(null);
  const [lastScrollY, setLastScrollY] = useState<number>(0);

  // 监听滚动事件以更新滚动方向
  useEffect(() => {
    // 滚动处理函数
    const scrollHandler = () => {
      const currentScrollY = window.scrollY;

      // 判断滚动方向并更新状态
      if (currentScrollY > lastScrollY) {
        if (scrollDir !== "down") {
          setScrollDir("down");
        }
      } else if (currentScrollY < lastScrollY) {
        if (scrollDir !== "up") {
          setScrollDir("up");
        }
      }

      // 如果滚动位置为顶部，重置滚动方向
      if (currentScrollY === 0) {
        if (scrollDir !== null) {
          setScrollDir(null);
        }
      }

      setLastScrollY(currentScrollY);
    };

    window.addEventListener("scroll", scrollHandler);
    return () => {
      window.removeEventListener("scroll", scrollHandler);
    };
  }, [lastScrollY, scrollDir]);

  // 根据滚动方向设置头部的类名
  let headerClass;
  if (scrollDir === "down") {
    headerClass = "top-0 left-0 right-0";
  } else if (scrollDir === "up") {
    headerClass =
      "top-8 left-[calc(50vw-486px)] right-[calc(50vw-486px)] rounded-3xl min-w-[500px] max-[1024px]:left-1/10 max-[1024px]:right-1/10";
  } else {
    headerClass = "top-0 left-0 right-0";
  }

  return (
    <>
      <ScrollProgress />
      <header
        className={`${headerClass} group flex justify-center fixed top-0 left-0 right-0 items-center z-10 bg-[rgba(230,230,230,0.3)] dark:bg-[rgba(11,11,15,0.3)] backdrop-blur-md shadow-sm border-b border-[rgba(180,180,180,0.2)] border-[1] h-[var(--header-h)] p-2.5 transition-[top right left] duration-[400ms] ease-in-out`}
      >
        <div className="flex w-4xl max-[1024px]:w-2xl max-[896px]:w-[460px] h-full pt-1 pb-1 items-center justify-between">
          <div className="flex flex-1 justify-start"></div>
          <div className="h-full flex items-center relative flex-row justify-center">
            <NavigateBar />
          </div>
          <div className="flex flex-1 items-center justify-end">
            <SearchBar placeholder="search..."></SearchBar>
          </div>
        </div>
      </header>
    </>
  );
}
