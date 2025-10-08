import ThemeToggle from "./ThemeToggle";
import CursorToggle from "./CursorToggle";

export default function Toolbar() {
  return (
    <div
      className={`bottom-40 right-2 fixed transition-transform duration-[400ms] ease-in-out`}
    >
      <ThemeToggle />
      <CursorToggle />
    </div>
  );
}
