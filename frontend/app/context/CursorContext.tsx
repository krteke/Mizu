"use client";

import {
  createContext,
  Dispatch,
  SetStateAction,
  ReactNode,
  useState,
  useEffect,
} from "react";

export type CursorType = {
  mode: "default" | "line" | "wrap";
  elementInfo?: TargetElementInfo | null;
  textHeight?: number;
};

export interface TargetElementInfo {
  width: number;
  height: number;
  top: number;
  left: number;
  borderRadius?: string;
}

interface CursorContextType {
  cursorType: CursorType;
  setCursorType: Dispatch<SetStateAction<CursorType>>;
  isCustomCursor: boolean;
  setIsCustomCursor: Dispatch<SetStateAction<boolean>>;
}

export const CursorContext = createContext<CursorContextType | undefined>(
  undefined
);

export const CursorProvider = ({ children }: { children: ReactNode }) => {
  const [cursorType, setCursorType] = useState<CursorType>({ mode: "default" });
  const [isCustomCursor, setIsCustomCursor] = useState(true);

  useEffect(() => {
    const customCursor = localStorage.getItem("user-custom-cursor");

    if (customCursor) {
      setIsCustomCursor(JSON.parse(customCursor));
    } else {
      setIsCustomCursor(false);
    }
  }, []);

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
