"use client";

import { useContext } from "react";
import MagneticElement from "./MagneticElement";
import { CursorContext } from "../context/CursorContext";
import DefaultCursor from "../assets/default-cursor.svg";
import CustomCursor from "../assets/custom-cursor.svg";

export default function CursorToggle() {
  const context = useContext(CursorContext);
  if (!context) {
    return;
  }

  const { isCustomCursor, setIsCustomCursor } = context;

  function changeCursor() {
    setIsCustomCursor((prev) => !prev);
  }

  return (
    <MagneticElement mode="wrap">
      <button
        onClick={changeCursor}
        className="relative flex h-9 w-9 rounded-[44%] justify-center items-center cursor-pointer shadow-button bg-[#d0d0d0] dark:bg-[#848484] border-none top-[30px] transition-[box-shadow transform] duration-[400ms] ease-in-out hover:translate-y-[-2px] hover:shadow-button-hover hover:scale-105"
      >
        <div
          className={`${
            isCustomCursor ? "opacity-100 scale-100" : " opacity-0 scale-0"
          } absolute w-7 h-7 top-1/2 left-1/2 translate-y-[-50%] translate-x-[-50%] pointer-events-none transition-[opacity scale] duration-200 ease-in-out`}
        >
          <CustomCursor />
        </div>
        <div
          className={`${
            isCustomCursor ? "opacity-0 scale-0" : "opacity-100 scale-100"
          } absolute w-7 h-7 top-1/2 left-1/2 translate-y-[-50%] translate-x-[-50%] pointer-events-none transition-[opacity scale] duration-200 ease-in-out`}
        >
          <DefaultCursor />
        </div>
      </button>
    </MagneticElement>
  );
}
