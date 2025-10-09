"use client";

import { useState } from "react";
import MagneticElement from "./MagneticElement";
import { AnimatePresence, motion } from "motion/react";
import Add from "../assets/add.svg";
import ThemeToggle from "./ThemeToggle";
import ScrollToEdge from "./ScrollToEdge";
import CursorToggle from "./CursorToggle";

export default function FloatingMenu() {
  const [isOpen, setIsOpen] = useState(false);

  // 定义父容器和子项的动画变体
  const menuVariants = {
    hidden: {
      opacity: 0,
      transition: {
        when: "afterChildren",
        staggerChildren: 0.05,
        staggerDirection: -1,
      },
    },
    visible: {
      opacity: 1,
      transition: {
        when: "beforeChildren",
        staggerChildren: 0.1,
      },
    },
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 15 },
    visible: { opacity: 1, y: 0 },
  };

  return (
    <div className="fixed bottom-20 right-2 flex justify-center items-center">
      <AnimatePresence>
        {isOpen && (
          <motion.div
            variants={menuVariants}
            initial="hidden"
            animate="visible"
            exit="hidden"
            className="absolute flex flex-col bottom-full pb-2.5"
          >
            <motion.div variants={itemVariants} className="my-1">
              <ThemeToggle />
            </motion.div>
            <motion.div variants={itemVariants} className="my-1">
              <ScrollToEdge dir="top" />
            </motion.div>
            <motion.div variants={itemVariants} className="my-1">
              <ScrollToEdge dir="bottom" />
            </motion.div>
            <motion.div variants={itemVariants} className="my-1">
              <CursorToggle />
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
      <MagneticElement mode="wrap">
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="relative transition-transform ease-in-out h-9 w-9 rounded-[44%] bg-[#d0d0d0] dark:bg-[#848484] hover:scale-105 cursor-pointer"
        >
          <div
            className="absolute w-8 h-8 top-1/2 left-1/2 translate-y-[-50%] translate-x-[-50%] pointer-events-none transition-all duration-200 ease-in-out"
            style={{ transform: isOpen ? "rotate(45deg)" : "rotate(0deg)" }}
          >
            <Add />
          </div>
        </button>
      </MagneticElement>
    </div>
  );
}
