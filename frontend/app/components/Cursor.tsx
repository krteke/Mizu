"use client";

import { useContext, useEffect } from "react";
import { motion, useMotionValue, useSpring } from "framer-motion";
import { CursorContext } from "../context/CursorContext";

// 鼠标光标组件，显示自定义的光标和跟随效果
export default function Cursor() {
  // 获取光标上下文
  const context = useContext(CursorContext);
  // 确保组件在 CursorProvider 内使用
  if (!context) {
    throw new Error("Cursor must be used within a CursorProvider");
  }
  // 获取上下文中的 cursorType
  const { cursorType } = context;

  // 使用 Framer Motion 的 useMotionValue 和 useSpring 创建平滑的动画效果
  const springConfig = { damping: 25, stiffness: 500 };
  //-- 主光标 --//
  const primaryMouseX = useMotionValue(-100);
  const primaryMouseY = useMotionValue(-100);
  const width = useMotionValue(25);
  const height = useMotionValue(25);
  const primaryWidth = useMotionValue(15);
  const primaryHeight = useMotionValue(15);
  const borderRadius = useMotionValue("50%");
  const backgroundColor = useMotionValue("gray");
  const scale = useMotionValue(1);

  //-- 跟随光标 --//
  const followMouseScale = useMotionValue(1);
  const followerMouseSmoothX = useSpring(primaryMouseX, springConfig);
  const followerMouseSmoothY = useSpring(primaryMouseY, springConfig);
  const smoothWidth = useSpring(width, springConfig);
  const smoothHeight = useSpring(height, springConfig);
  const smoothPrimaryWidth = useSpring(primaryWidth, springConfig);
  const smoothPrimaryHeight = useSpring(primaryHeight, springConfig);
  const smoothBorderRadius = useSpring(borderRadius, {
    damping: 20,
    stiffness: 300,
  });
  const smoothBackgroundColor = useSpring(backgroundColor, springConfig);
  const smoothScale = useSpring(scale, springConfig);
  const smoothFollowerScale = useSpring(followMouseScale, springConfig);

  // 监听鼠标移动、按下和抬起事件以更新光标位置和状态
  useEffect(() => {
    const handleMouseMove = (event: MouseEvent) => {
      if (cursorType.mode === "default" || cursorType.mode === "line") {
        primaryMouseX.set(event.clientX);
        primaryMouseY.set(event.clientY);
      }
    };

    window.addEventListener("mousemove", handleMouseMove);

    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
    };
  }, [cursorType.mode, primaryMouseX, primaryMouseY]);

  // 监听鼠标按下和抬起事件以更新光标缩放状态
  useEffect(() => {
    // 鼠标按下时缩小光标
    const handleMouseDown = () => {
      if (cursorType.mode === "default") {
        followMouseScale.set(0.8);
        scale.set(0.9);
      }
    };

    // 鼠标抬起时恢复光标大小
    const handleMouseUp = () => {
      if (cursorType.mode === "default") {
        followMouseScale.set(1);
        scale.set(1);
      }
    };

    window.addEventListener("mousedown", handleMouseDown);
    window.addEventListener("mouseup", handleMouseUp);

    return () => {
      window.removeEventListener("mousedown", handleMouseDown);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [cursorType.mode, followMouseScale, scale]);

  // 根据 cursorType 的变化更新光标的外观和行为
  useEffect(() => {
    if (cursorType.mode === "wrap" && cursorType.elementInfo) {
      // 获取元素的信息并设置光标样式
      const {
        width: elWidth,
        height: elHeight,
        top: elTop,
        left: elLeft,
        borderRadius: elBorderRadius,
      } = cursorType.elementInfo;

      scale.set(0);
      followMouseScale.set(1);

      primaryMouseX.set(elLeft + elWidth / 2);
      primaryMouseY.set(elTop + elHeight / 2);

      width.set(elWidth + 10);
      height.set(elHeight + 10);
      if (typeof elBorderRadius === "string") {
        borderRadius.set(elBorderRadius);
      }
    } else if (cursorType.mode === "line" && cursorType.textHeight) {
      const textHeight = cursorType.textHeight;

      followMouseScale.set(0);
      primaryWidth.set(5);
      primaryHeight.set(textHeight);
      borderRadius.set("30%");
    } else {
      scale.set(1);
      width.set(25);
      height.set(25);
      primaryHeight.set(15);
      primaryWidth.set(15);
      borderRadius.set("50%");
      followMouseScale.set(1);
    }
  }, [
    primaryHeight,
    primaryWidth,
    followMouseScale,
    scale,
    cursorType,
    primaryMouseX,
    primaryMouseY,
    width,
    height,
    borderRadius,
    backgroundColor,
  ]);

  return (
    <>
      <motion.div
        className=" dark:bg-white bg-gray-800 border-gray-500"
        style={{
          position: "fixed",
          pointerEvents: "none",
          zIndex: 9999,
          left: primaryMouseX,
          top: primaryMouseY,
          translateX: "-50%",
          translateY: "-50%",
          scale: smoothScale,
          width: smoothPrimaryWidth,
          height: smoothPrimaryHeight,
          borderRadius: smoothBorderRadius,
        }}
      />
      <motion.div
        style={{
          position: "fixed",
          pointerEvents: "none",
          zIndex: 9998,
          left: followerMouseSmoothX,
          top: followerMouseSmoothY,
          translateX: "-50%",
          translateY: "-50%",
          width: smoothWidth,
          height: smoothHeight,
          opacity: 0.5,
          borderRadius: smoothBorderRadius,
          backgroundColor: smoothBackgroundColor,
          scale: smoothFollowerScale,
        }}
        transition={{ type: "spring", ...springConfig }}
      />
    </>
  );
}
