"use client";

import { useEffect, useState } from "react";

// 判断是否为移动设备的 Hook
export const useIsMobile = () => {
  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    const checkDevice = () => {
      const isMobileDevice = window.matchMedia("(pointer: coarse)").matches;
      setIsMobile(isMobileDevice);
    };
    checkDevice();
  }, []);

  return isMobile;
};
