"use client";

import React, { ReactNode, useContext, useEffect } from "react";
import { useIsMobile } from "../hooks/useIsMobile";
import { CursorContext, CursorProvider } from "../context/CursorContext";
import Cursor from "./Cursor";
import { ThemeProvider } from "next-themes";

// 客户端提供者组件，包装应用以提供主题和自定义光标功能
export default function ClientProviders({ children }: { children: ReactNode }) {
  return (
    <CursorProvider>
      <ThemeProvider storageKey="theme" attribute={"class"}>
        <CursorManager>{children}</CursorManager>
      </ThemeProvider>
    </CursorProvider>
  );
}

// 管理自定义光标显示的组件
function CursorManager({ children }: { children: React.ReactNode }) {
  // 获取光标上下文和是否为移动设备的状态
  const context = useContext(CursorContext);
  const isMobile = useIsMobile();

  // 根据 isCustomCursor 状态添加或移除隐藏原生光标的类名
  const isCustomCursor = context?.isCustomCursor;

  // 当 isCustomCursor 或 isMobile 变化时，更新 body 的类名以隐藏或显示原生光标
  useEffect(() => {
    if (isCustomCursor) {
      document.body.classList.add("hide-native-cursor");
    } else {
      document.body.classList.remove("hide-native-cursor");
    }
  }, [isCustomCursor]);

  return (
    <>
      {isCustomCursor && !isMobile && <Cursor />}
      {children}
    </>
  );
}
