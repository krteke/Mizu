"use client";

import { useContext, useEffect } from "react";
import { motion, useMotionValue, useSpring } from "framer-motion";
import { CursorContext } from "../context/cursor_context";

export default function Cursor() {
  const context = useContext(CursorContext);
  if (!context) {
    throw new Error("Cursor must be used within a CursorProvider");
  }
  const { cursorType } = context;

  const springConfig = { damping: 25, stiffness: 500 };
  const primaryMouseX = useMotionValue(-100);
  const primaryMouseY = useMotionValue(-100);
  const width = useMotionValue(25);
  const height = useMotionValue(25);
  const primaryWidth = useMotionValue(15);
  const primaryHeight = useMotionValue(15);
  const borderRadius = useMotionValue("50%");
  const backgroundColor = useMotionValue("gray");
  const scale = useMotionValue(1);
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

  useEffect(() => {
    const handleMouseDown = () => {
      if (cursorType.mode === "default") {
        followMouseScale.set(0.8);
        scale.set(0.9);
      }
    };

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

  useEffect(() => {
    if (cursorType.mode === "wrap" && cursorType.elementInfo) {
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
      // backgroundColor.set("gray");
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
      // backgroundColor.set("gray");
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
        // transition={{ type: "spring", ...springConfig }}
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
          // mixBlendMode: "difference",
        }}
        transition={{ type: "spring", ...springConfig }}
      />
    </>
  );
}
