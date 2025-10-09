"use client";
import { flushSync } from "react-dom";
import { useTheme } from "next-themes";
import Sun from "../assets/sun.svg";
import Moon from "../assets/moon.svg";
import { useEffect, useState } from "react";
import MagneticElement from "./MagneticElement";

// 一个切换主题的按钮组件
export default function ThemeToggle() {
  // 使用 useTheme 来获取和设置主题
  const { theme, setTheme, resolvedTheme } = useTheme();

  const [mounted, setMounted] = useState(false);

  // 组件挂载后设置 mounted 为 true
  useEffect(() => {
    setMounted(true);
  }, []);

  // 如果组件还未挂载，返回一个占位的 div
  if (!mounted) {
    return <div className="w-9 h-9"></div>;
  }

  // 根据主题设置 isDark
  let isDark = resolvedTheme === "dark";
  // 切换主题的函数
  function changeTheme() {
    isDark = !isDark;
    setTheme(theme === "light" ? "dark" : "light");
  }

  // 使用 View Transition API 来实现主题切换的动画效果
  function updateView(event: React.MouseEvent<HTMLButtonElement>) {
    // 如果浏览器不支持 View Transition API，直接切换主题
    if (!document.startViewTransition) {
      changeTheme();
      return;
    }

    // 使用 View Transition API 来切换主题
    const transition = document.startViewTransition(() => {
      // 使用 flushSync 确保在视图切换期间同步更新主题
      flushSync(() => {
        changeTheme();
      });
    });

    // 等待视图切换准备好后，执行圆形扩展动画
    transition.ready.then(() => {
      const x = event.clientX;
      const y = event.clientY;

      // 计算圆形扩展的终点半径
      const endRadius = Math.hypot(
        Math.max(x, innerWidth - x),
        Math.max(y, innerHeight - y)
      );

      // const isTransitioningToDark =
      // theme === "light" || (theme === "system" && resolvedTheme === "light");

      // 定义 clip-path 动画的起点和终点
      const clipPath = [
        `circle(0px at ${x}px ${y}px)`,
        `circle(${endRadius}px at ${x}px ${y}px)`,
      ];

      // 对 document.documentElement 执行动画
      document.documentElement.animate(
        {
          clipPath: isDark ? [...clipPath].reverse() : clipPath,
        },
        {
          duration: 400,
          easing: "ease-in-out",
          pseudoElement: isDark
            ? "::view-transition-old(root)"
            : "::view-transition-new(root)",
        }
      );
    });
  }

  return (
    <MagneticElement mode="wrap">
      <button
        className="bg-[#d0d0d0] dark:bg-[#848484] relative flex justify-center shadow-button w-9 h-9 rounded-[44%] cursor-pointer transition-transform duration-[400ms] ease-in-out transform-gpu hover:scale-105"
        id="theme-toggle-button"
        onClick={updateView}
      >
        <div className=" absolute w-7 h-7 top-1/2 left-1/2 translate-y-[-50%] translate-x-[-50%] pointer-events-none">
          {isDark ? <Moon /> : <Sun />}
        </div>
      </button>
    </MagneticElement>
  );
}
