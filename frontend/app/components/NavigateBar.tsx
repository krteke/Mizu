import Link from "next/link";

export default function NavigateBar() {
  return (
    <>
      <nav className="">
        <Link href={"/articles"}>文章</Link>
      </nav>
      <nav className="">
        <Link href={"/notes"}>笔记</Link>
      </nav>
      <nav className="">
        <Link href={"/pictures"}>图片</Link>
      </nav>
      <nav className="">
        <Link href={"/talks"}>说说</Link>
      </nav>
      <nav className="">
        <Link href={"/thinks"}>思考</Link>
      </nav>
    </>
  );
}
