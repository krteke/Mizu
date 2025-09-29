import Link from "next/link";

export default function NotFound() {
  return (
    <div className="flex flex-col w-full h-[calc(100dvh-var(--footer-h))] pt-[var(--header-h)]">
      <h2>Not Found</h2>
      <p>Could not find requested resource</p>
      <Link href="/">Return Home</Link>
    </div>
  );
}
