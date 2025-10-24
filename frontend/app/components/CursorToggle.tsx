"use client";

import { useContext } from "react";
import MagneticElement from "./MagneticElement";
import { CursorContext } from "../context/CursorContext";
import DefaultCursor from "../assets/default-cursor.svg";
import CustomCursor from "../assets/custom-cursor.svg";

// 一个切换光标样式的按钮组件
export default function CursorToggle({ className }: { className?: string }) {
  // 使用 CursorContext 来获取和设置光标状态
  const context = useContext(CursorContext);
  if (!context) {
    return;
  }

  // 解构出 isCustomCursor 和 setIsCustomCursor
  const { isCustomCursor, setIsCustomCursor } = context;

  // 切换光标状态的函数
  function changeCursor() {
    setIsCustomCursor((prev) => !prev);
  }

  return (
    <MagneticElement mode="wrap">
      <button onClick={changeCursor} className={className}>
        <div
          className={`${
            isCustomCursor ? "opacity-100 scale-100" : " opacity-0 scale-0"
          } absolute w-7 h-7 top-1/2 left-1/2 translate-y-[-50%] translate-x-[-50%] pointer-events-none transition-[opacity scale] duration-200 ease-in-out`}
        >
          <CustomCursor />
        </div>
        <div
          className={`${
            isCustomCursor ? "opacity-0 scale-0" : "opacity-100 scale-100"
          } absolute w-7 h-7 top-1/2 left-1/2 translate-y-[-50%] translate-x-[-50%] pointer-events-none transition-[opacity scale] duration-200 ease-in-out`}
        >
          <DefaultCursor />
        </div>
      </button>
    </MagneticElement>
  );
}
