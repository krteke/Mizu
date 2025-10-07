import { useState } from "react";
import ScrollProgress from "./scroll-progress";
import ScrollToBottom from "./scroll-to-bottom";
import ThemeToggle from "./theme-toggle";
import CursorToggle from "./cursor-toggle";

export default function Toolbar() {
  const [atTop, setAtTop] = useState(true);

  const scrollPercentHandler = (isAtTop: boolean) => {
    setAtTop(isAtTop);
  };

  return (
    <div
      className={`${
        atTop ? "translate-x-24" : "translate-0"
      } bottom-40 right-2 fixed transition-transform duration-[400ms] ease-in-out`}
    >
      <ThemeToggle />
      <ScrollProgress onScrollToTop={scrollPercentHandler} />
      <ScrollToBottom />
      <CursorToggle />
    </div>
  );
}
