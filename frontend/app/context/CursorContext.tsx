"use client";

import {
  createContext,
  Dispatch,
  SetStateAction,
  ReactNode,
  useState,
  useEffect,
} from "react";

// 定义光标类型
export type CursorType = {
  mode: "default" | "line" | "wrap";
  elementInfo?: TargetElementInfo | null;
  textHeight?: number;
};

// 定义目标元素信息
export interface TargetElementInfo {
  width: number;
  height: number;
  top: number;
  left: number;
  borderRadius?: string;
}

// 定义光标上下文类型
interface CursorContextType {
  cursorType: CursorType;
  setCursorType: Dispatch<SetStateAction<CursorType>>;
  isCustomCursor: boolean;
  setIsCustomCursor: Dispatch<SetStateAction<boolean>>;
}

// 创建光标上下文
export const CursorContext = createContext<CursorContextType | undefined>(
  undefined
);

// 光标提供者组件
export const CursorProvider = ({ children }: { children: ReactNode }) => {
  const [cursorType, setCursorType] = useState<CursorType>({ mode: "default" });
  const [isCustomCursor, setIsCustomCursor] = useState(true);

  // 从 localStorage 获取用户的光标偏好设置, 如果没有则设置为false, 即默认光标
  useEffect(() => {
    const customCursor = localStorage.getItem("user-custom-cursor");

    if (customCursor) {
      setIsCustomCursor(JSON.parse(customCursor));
    } else {
      setIsCustomCursor(false);
    }
  }, []);

  // 当 isCustomCursor 变化时, 将其存储到 localStorage 中
  useEffect(() => {
    localStorage.setItem("user-custom-cursor", JSON.stringify(isCustomCursor));
  }, [isCustomCursor]);

  return (
    <CursorContext.Provider
      value={{ cursorType, setCursorType, isCustomCursor, setIsCustomCursor }}
    >
      {children}
    </CursorContext.Provider>
  );
};
