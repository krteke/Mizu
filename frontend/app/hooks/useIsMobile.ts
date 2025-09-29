"use client";

import { useEffect, useState } from "react";

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
