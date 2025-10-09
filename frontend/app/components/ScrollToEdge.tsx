"use client";
import { scrollDir } from "@/types/types";
import Bottom from "../assets/bottom.svg";
import Top from "../assets/top.svg";
import MagneticElement from "./MagneticElement";

export default function ScrollToEdge({ dir }: { dir: scrollDir }) {
  // 滑动到指定方向
  function scrollToDir(direction: scrollDir) {
    window.scrollTo({
      top: direction === "top" ? 0 : document.body.scrollHeight,
      behavior: "smooth",
    });
  }

  return (
    <MagneticElement mode="wrap">
      <button
        onClick={() => scrollToDir(dir)}
        className="relative flex h-9 w-9 rounded-[44%] justify-center items-center bg-[#d0d0d0] dark:bg-[#848484] border-none transition-transform duration-[400ms] ease-in-out hover:scale-105 cursor-pointer"
      >
        <div className=" absolute w-7 h-7 top-1/2 left-1/2 translate-y-[-50%] translate-x-[-50%] pointer-events-none">
          {dir === "top" ? <Top /> : <Bottom />}
        </div>
      </button>
    </MagneticElement>
  );
}
