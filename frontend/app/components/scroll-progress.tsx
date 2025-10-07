"use client";
import { useEffect, useState } from "react";
import Top from "../assets/top.svg";
import MagneticElement from "./magnetic-element";

type Props = {
  onScrollToTop: (isAtTop: boolean) => void;
};

export default function ScrollProgress({ onScrollToTop }: Props) {
  const [scrollPercent, setScrollPercent] = useState(0);

  useEffect(() => {
    const scrollHandler = () => {
      const scroll_top = document.scrollingElement?.scrollTop || 0;
      const scroll_height = document.scrollingElement?.scrollHeight || 0;
      const client_height = document.scrollingElement?.clientHeight || 0;
      const percent = (scroll_top / (scroll_height - client_height)) * 100;
      setScrollPercent(percent);
    };

    window.addEventListener("scroll", scrollHandler);

    if (scrollPercent === 0) {
      onScrollToTop(true);
    } else {
      onScrollToTop(false);
    }
    return () => {
      window.removeEventListener("scroll", scrollHandler);
    };
  }, [onScrollToTop, scrollPercent]);

  function scrollToTop() {
    window.scrollTo({ top: 0, behavior: "smooth" });
  }

  return (
    <MagneticElement mode="wrap">
      <button
        onClick={scrollToTop}
        className="group bg-[#d0d0d0] dark:bg-[#848484] relative flex justify-center shadow-button w-9 h-9 top-2.5 rounded-[44%] border-none cursor-pointer transition-[box-shadow transform] duration-[400ms] ease-in-out transform-gpu hover:translate-y-[-2px] hover:shadow-button-hover hover:scale-105"
      >
        <div
          className={`absolute top-1/2 left-1/2 translate-x-[-50%] translate-y-[-50%] text-[16px] text-[#111111] dark:text-[#d4d4d4] transition-[opacity transform] duration-200 ease-in-out font-bold group-hover:scale-0 group-hover:opacity-0 ${
            scrollPercent < 100 ? "opacity-100 scale-100" : "opacity-0 scale-0"
          }`}
        >
          {Math.round(scrollPercent)}
        </div>
        <div
          className={` absolute top-1/2 left-1/2 translate-x-[-50%] translate-y-[-50%] transition-[scale transform] duration-200 ease-in-out w-7 h-7 group-hover:scale-100 group-hover:opacity-100 ${
            scrollPercent < 100 ? "opacity-0 scale-0" : "opacity-100 scale-100"
          }`}
        >
          <Top />
        </div>
      </button>
    </MagneticElement>
  );
}
