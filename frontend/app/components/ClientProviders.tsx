"use client";

import React, { ReactNode, useContext, useEffect } from "react";
import { useIsMobile } from "../hooks/useIsMobile";
import { CursorContext, CursorProvider } from "../context/CursorContext";
import Cursor from "./Cursor";
import { ThemeProvider } from "next-themes";

export default function ClientProviders({ children }: { children: ReactNode }) {
  return (
    <CursorProvider>
      <ThemeProvider storageKey="theme" attribute={"class"}>
        <CursorManager>{children}</CursorManager>
      </ThemeProvider>
    </CursorProvider>
  );
}

function CursorManager({ children }: { children: React.ReactNode }) {
  const context = useContext(CursorContext);
  const isMobile = useIsMobile();

  const isCustomCursor = context?.isCustomCursor;

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
