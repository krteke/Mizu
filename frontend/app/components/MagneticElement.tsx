"use client";

import { useMotionValue, useSpring, motion } from "framer-motion";
import React, { ReactElement, useContext, useRef } from "react";
import { CursorContext } from "../context/CursorContext";

// 磁性元素组件，提供鼠标悬停时的磁性吸附效果
type MagneticElementProps = {
  children: ReactElement;
  strength?: { x: number; y: number };
  mode?: "magnetic" | "wrap" | "line";
};

// 磁性元素组件
const MagneticElement = ({
  children,
  strength = { x: 0.2, y: 0.3 },
  mode = "magnetic",
}: MagneticElementProps) => {
  // 引用元素和上下文
  const ref = useRef<HTMLDivElement>(null);
  const context = useContext(CursorContext);

  // 确保组件在 CursorProvider 内使用
  if (!context) {
    throw new Error("MagneticElement must be used within a CursorProvider");
  }

  // 获取上下文中的 setCursorType 方法
  const { setCursorType } = context;

  // 使用 Framer Motion 的 useMotionValue 和 useSpring 创建平滑的动画效果
  const x = useMotionValue(0);
  const y = useMotionValue(0);

  const springConfig = { damping: 15, stiffness: 150 };
  const smoothx = useSpring(x, springConfig);
  const smoothy = useSpring(y, springConfig);

  // 处理鼠标移动、进入和离开事件
  const handleMouseMove = (event: React.MouseEvent<HTMLDivElement>) => {
    // 如果模式不是 "line" 且引用存在，计算鼠标位置并更新 x 和 y
    if (mode !== "line" && ref.current) {
      const { left, top, width, height } = ref.current.getBoundingClientRect();
      const mouseX = event.clientX - (left + width / 2);
      const mouseY = event.clientY - (top + height / 2);
      x.set(mouseX * strength.x);
      y.set(mouseY * strength.y);
    }
  };

  // 鼠标离开时重置状态
  const handleMouseLeave = () => {
    setCursorType({ mode: "default" });
    x.set(0);
    y.set(0);
  };

  // 鼠标进入时根据模式设置不同的光标类型
  const handleMouseEnter = () => {
    if (!ref.current) return;

    if (mode === "wrap") {
      // 获取子元素的信息并设置光标类型为 "wrap"
      const targetElement = ref.current.firstElementChild as HTMLElement;

      if (!targetElement) return;
      const { width, height, top, left } =
        targetElement.getBoundingClientRect();
      const borderRadius = window.getComputedStyle(targetElement).borderRadius;

      setCursorType({
        mode: "wrap",
        elementInfo: { width, height, top, left, borderRadius },
      });
    } else if (mode === "line") {
      // 获取子元素的字体大小并设置光标类型为 "line"
      const targetElement = ref.current.firstElementChild as HTMLElement;

      if (!targetElement) return;
      const fontSizeStr = window.getComputedStyle(targetElement).fontSize;
      const fontSize = parseFloat(fontSizeStr);

      setCursorType({
        mode: "line",
        textHeight: fontSize,
      });
    }
  };

  return (
    <motion.div
      ref={ref}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
      onMouseEnter={handleMouseEnter}
      style={{
        x: smoothx,
        y: smoothy,
      }}
      transition={{ type: "spring", ...springConfig }}
    >
      {children}
    </motion.div>
  );
};

export default MagneticElement;
