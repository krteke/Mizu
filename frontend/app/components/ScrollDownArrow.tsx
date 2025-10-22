"use client";

import { motion } from "framer-motion";

interface ScrollDownArrowProps {
  onClick?: () => void;
}

export default function ScrollDownArrow({ onClick }: ScrollDownArrowProps) {
  const handleClick = () => {
    if (onClick) {
      onClick();
    } else {
      // 默认滚动一个视口高度
      window.scrollTo({
        top: window.innerHeight,
        behavior: "smooth",
      });
    }
  };

  return (
    <motion.button
      onClick={handleClick}
      className="absolute bottom-8 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2 cursor-pointer group"
      initial={{ opacity: 0, y: -20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: 1, duration: 0.8 }}
      aria-label="向下滚动"
    >
      <motion.svg
        className="w-8 h-8 text-gray-600 dark:text-gray-400 group-hover:text-gray-900 dark:group-hover:text-gray-100 transition-colors"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
        animate={{ y: [0, 10, 0] }}
        transition={{
          duration: 1.5,
          repeat: Infinity,
          ease: "easeInOut",
        }}
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M19 14l-7 7m0 0l-7-7m7 7V3"
        />
      </motion.svg>
    </motion.button>
  );
}
