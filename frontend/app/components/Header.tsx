"use client";

import { useEffect, useRef, useState } from "react";
import SearchBar from "./SearchBar";
import ScrollProgress from "./ScrollProgress";
import NavigateBar from "./NavigateBar";
import { containerVariants, headerVariants } from "./header.cva";
import { VariantProps } from "class-variance-authority";

type HeaderState = VariantProps<typeof headerVariants>["state"];

export default function Header() {
  const [headerState, setHeaderState] = useState<HeaderState>("fullWidth");

  const lastScrollY = useRef<number>(0); // 上一次的滚动位置

  // 监听滚动事件以更新滚动方向
  useEffect(() => {
    // 滚动处理函数
    const scrollHandler = () => {
      const currentScrollY = window.scrollY;

      // 判断滚动方向并更新状态
      let newState: HeaderState = "fullWidth";
      if (currentScrollY < 10) {
        newState = "fullWidth";
      } else if (currentScrollY > lastScrollY.current) {
        newState = "fullWidth";
      } else {
        newState = "shrunken";
      }

      setHeaderState((prevState) => {
        if (prevState !== newState) {
          return newState;
        }
        return prevState;
      });

      lastScrollY.current = currentScrollY;
    };

    window.addEventListener("scroll", scrollHandler);

    return () => {
      window.removeEventListener("scroll", scrollHandler);
    };
  }, []);

  return (
    <>
      <ScrollProgress />
      <header className={headerVariants({ state: headerState })}>
        <div className={containerVariants()}>
          <div className="flex-1 justify-start"></div>
          <div className="flex items-center justify-center">
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
