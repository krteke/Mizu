"use client";
import { scrollDir } from "@/types/types";
import Bottom from "../assets/bottom.svg";
import Top from "../assets/top.svg";
import MagneticElement from "./MagneticElement";

export default function ScrollToEdge({
  dir,
  className,
}: {
  dir: scrollDir;
  className?: string;
}) {
  // 滑动到指定方向
  function scrollToDir(direction: scrollDir) {
    window.scrollTo({
      top: direction === "top" ? 0 : document.body.scrollHeight,
      behavior: "smooth",
    });
  }

  return (
    <MagneticElement mode="wrap">
      <button onClick={() => scrollToDir(dir)} className={className}>
        <div className=" absolute w-7 h-7 top-1/2 left-1/2 translate-y-[-50%] translate-x-[-50%] pointer-events-none">
          {dir === "top" ? <Top /> : <Bottom />}
        </div>
      </button>
    </MagneticElement>
  );
}
