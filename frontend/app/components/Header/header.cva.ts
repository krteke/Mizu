import { cva } from "class-variance-authority";

export const headerVariants = cva(
  [
    "flex justify-center items-center fixed z-100 h-[var(--header-h)] p-2.5",
    "bg-gray-200/50 dark:bg-zinc-900/50 backdrop-blur-lg",
    "border-b border-black/10 dark:border-white/10 shadow-sm",
    "transition-all duration-[400ms] ease-in-out",
  ],
  {
    variants: {
      state: {
        fullWidth: "top-0 left-0 right-0",
        shrunken: [
          "top-8 rounded-3xl shadow-md",
          "left-[calc(50vw-486px)] z-100 right-[calc(50vw-486px)] min-w-[500px] max-[1024px]:left-1/10 max-[1024px]:right-1/10",
        ],
      },
    },
    defaultVariants: {
      state: "fullWidth",
    },
  },
);

export const containerVariants = cva([
  "flex w-4xl max-[1024px]:w-2xl max-[896px]:w-[460px] h-full pt-1 pb-1 items-center justify-between",
]);
