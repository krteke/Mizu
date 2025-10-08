"use client";
import Bottom from "../assets/bottom.svg";
import MagneticElement from "./MagneticElement";

export default function ScrollToBottom() {
  function scrollToBottom() {
    window.scrollTo({ top: document.body.scrollHeight, behavior: "smooth" });
  }

  return (
    <MagneticElement mode="wrap">
      <button
        onClick={scrollToBottom}
        className="relative flex h-9 w-9 rounded-[44%] justify-center items-center cursor-pointer shadow-button bg-[#d0d0d0] dark:bg-[#848484] border-none top-5 transition-[box-shadow transform] duration-[400ms] ease-in-out hover:translate-y-[-2px] hover:shadow-button-hover hover:scale-105"
      >
        <div className=" absolute w-7 h-7 top-1/2 left-1/2 translate-y-[-50%] translate-x-[-50%] pointer-events-none">
          <Bottom />
        </div>
      </button>
    </MagneticElement>
  );
}
