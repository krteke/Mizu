"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useTheme } from "next-themes";

interface CodeBlockProps {
  html?: string;
  lightHtml?: string;
  darkHtml?: string;
  language?: string;
  maxHeight?: number;
  collapsible?: boolean;
  defaultCollapsed?: boolean;
  showLineNumbers?: boolean;
}

export default function CodeBlock({
  html,
  lightHtml,
  darkHtml,
  language = "code",
  maxHeight = 500,
  collapsible = true,
  defaultCollapsed = false,
  showLineNumbers = true,
}: CodeBlockProps) {
  const [isCollapsed, setIsCollapsed] = useState(defaultCollapsed);
  const [isCopied, setIsCopied] = useState(false);
  const { resolvedTheme } = useTheme();

  const displayHtml = (() => {
    if (lightHtml && darkHtml) {
      return resolvedTheme === "dark" ? darkHtml : lightHtml;
    }

    return html || "";
  })();

  const handleCopy = async () => {
    const tempDiv = document.createElement("div");
    tempDiv.innerHTML = displayHtml;
    const code = tempDiv.textContent || "";

    try {
      await navigator.clipboard.writeText(code);
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 2000);
    } catch (error) {
      console.error("Failed to copy code:", error);
    }
  };

  const toggleCollapse = () => {
    if (collapsible) {
      setIsCollapsed(!isCollapsed);
    }
  };

  return (
    <div className="relative group my-6 rounded-lg overflow-hidden border border-gray-200 dark:border-gray-800 shadow-lg">
      <div className="flex items-center justify-between px-4 py-2 bg-gray-100 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800">
        <div className="flex items-center gap-3">
          <div className="flex gap-2">
            <div className="w-3 h-3 rounded-full bg-red-500" />
            <div className="w-3 h-3 rounded-full bg-yellow-500" />
            <div className="w-3 h-3 rounded-full bg-green-500" />
          </div>

          <span className="text-xs font-mono text-gray-500 dark:text-gray-400 uppercase">
            {language}
          </span>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={handleCopy}
            className="p-2 rounded hover:bg-gray-200 dark:hover:bg-gray-800 transition-colors"
            title="复制代码"
          >
            {isCopied ? (
              <svg
                className="w-4 h-4 text-green-500"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M5 13l4 4L19 7"
                />
              </svg>
            ) : (
              <svg
                className="w-4 h-4 text-gray-600 dark:text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                />
              </svg>
            )}
          </button>

          {collapsible && (
            <button
              onClick={toggleCollapse}
              className="p-2 rounded hover:bg-gray-200 dark:hover:bg-gray-800 transition-colors"
              title={isCollapsed ? "展开" : "折叠"}
            >
              <motion.svg
                className="w-4 h-4 text-gray-600 dark:text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                animate={{ rotate: isCollapsed ? 0 : 180 }}
                transition={{ duration: 0.3 }}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M19 9l-7 7-7-7"
                />
              </motion.svg>
            </button>
          )}
        </div>
      </div>

      <AnimatePresence initial={false}>
        {!isCollapsed && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.3, ease: "easeInOut" }}
            className="overflow-hidden"
          >
            <div
              className="code-block-wrapper overflow-x-auto"
              style={{ maxHeight: `${maxHeight}px` }}
            >
              <div
                className={`code-block-content ${showLineNumbers ? "line-numbers" : ""}`}
                dangerouslySetInnerHTML={{ __html: displayHtml }}
              />
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {isCollapsed && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="px-4 py-3 bg-gray-50 dark:bg-gray-900/50 text-center text-sm text-gray-500 dark:text-gray-400 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-900 transition-colors"
          onClick={toggleCollapse}
        >
          点击展开代码 ({language})
        </motion.div>
      )}
    </div>
  );
}
