"use client";

import { useMotionValue, useSpring, motion } from "framer-motion";
import React, { ReactElement, useContext, useRef } from "react";
import { CursorContext } from "../context/cursor_context";

type MagneticElementProps = {
  children: ReactElement;
  strength?: { x: number; y: number };
  mode?: "magnetic" | "wrap" | "line";
};

const MagneticElement = ({
  children,
  strength = { x: 0.2, y: 0.3 },
  mode = "magnetic",
}: MagneticElementProps) => {
  const ref = useRef<HTMLDivElement>(null);
  const context = useContext(CursorContext);

  if (!context) {
    throw new Error("MagneticElement must be used within a CursorProvider");
  }

  const { setCursorType } = context;

  const x = useMotionValue(0);
  const y = useMotionValue(0);

  const springConfig = { damping: 15, stiffness: 150 };
  const smoothx = useSpring(x, springConfig);
  const smoothy = useSpring(y, springConfig);

  const handleMouseMove = (event: React.MouseEvent<HTMLDivElement>) => {
    if (mode !== "line" && ref.current) {
      const { left, top, width, height } = ref.current.getBoundingClientRect();
      const mouseX = event.clientX - (left + width / 2);
      const mouseY = event.clientY - (top + height / 2);
      x.set(mouseX * strength.x);
      y.set(mouseY * strength.y);
    }
  };

  const handleMouseLeave = () => {
    setCursorType({ mode: "default" });
    x.set(0);
    y.set(0);
  };

  const handleMouseEnter = () => {
    if (!ref.current) return;

    if (mode === "wrap") {
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
