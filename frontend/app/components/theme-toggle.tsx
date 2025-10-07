"use client";
import { flushSync } from "react-dom";
import { useTheme } from "next-themes";
import Sun from "../assets/sun.svg";
import Moon from "../assets/moon.svg";
import { useEffect, useState } from "react";
import MagneticElement from "./magnetic-element";

export default function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) {
    return <div className="w-9 h-9"></div>;
  }

  let isDark = theme === "dark";
  function changeTheme() {
    isDark = !isDark;
    setTheme(theme === "light" ? "dark" : "light");
  }

  function updateView(event: React.MouseEvent<HTMLButtonElement>) {
    if (!document.startViewTransition) {
      changeTheme();
      return;
    }

    const transition = document.startViewTransition(() => {
      flushSync(() => {
        changeTheme();
      });
    });

    transition.ready.then(() => {
      const x = event.clientX;
      const y = event.clientY;
      const endRadius = Math.hypot(
        Math.max(x, innerWidth - x),
        Math.max(y, innerHeight - y)
      );
      const clipPath = [
        `circle(0px at ${x}px ${y}px)`,
        `circle(${endRadius}px at ${x}px ${y}px)`,
      ];
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
        className="bg-[#d0d0d0] dark:bg-[#848484] relative flex justify-center shadow-button w-9 h-9 rounded-[44%] border-none cursor-pointer transition-[box-shadow transform] duration-[400ms] ease-in-out transform-gpu hover:translate-y-[-2px] hover:shadow-button-hover hover:scale-105"
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
