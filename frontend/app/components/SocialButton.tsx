"use client";

import { motion } from "framer-motion";
import MagneticElement from "./MagneticElement";

interface SocialButtonProps {
  href: string;
  icon: React.ReactNode;
  label: string;
  color?: string;
}

export default function SocialButton({
  href,
  icon,
  label,
  color = "#6366f1",
}: SocialButtonProps) {
  return (
    <MagneticElement strength={{ x: 0.3, y: 0.3 }}>
      <motion.a
        href={href}
        target="_blank"
        rel="noopener noreferrer"
        className="relative group flex items-center justify-center w-12 h-12 rounded-full bg-gray-100 dark:bg-gray-800 hover:bg-white dark:hover:bg-gray-700 transition-all duration-300 shadow-md hover:shadow-xl"
        whileHover={{ scale: 1.1 }}
        whileTap={{ scale: 0.95 }}
        aria-label={label}
      >
        <motion.div
          className="relative z-10"
          style={{ color }}
          whileHover={{ rotate: [0, -10, 10, -10, 0] }}
          transition={{ duration: 0.5 }}
        >
          {icon}
        </motion.div>

        {/* 悬停时显示标签 */}
        <motion.span
          className="absolute -bottom-8 left-1/2 -translate-x-1/2 px-2 py-1 bg-gray-900 dark:bg-gray-100 text-white dark:text-gray-900 text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none"
          initial={{ y: -5, opacity: 0 }}
          whileHover={{ y: 0, opacity: 1 }}
        >
          {label}
        </motion.span>
      </motion.a>
    </MagneticElement>
  );
}
