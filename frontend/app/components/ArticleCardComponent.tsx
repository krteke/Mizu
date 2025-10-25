import { ArticleCard } from "@/types/types";
import Link from "next/link";
import MagneticElement from "./MagneticElement";

export default function ArticleCardComponent({
  article,
  basePath,
}: {
  article: ArticleCard;
  basePath: string;
}) {
  return (
    <MagneticElement strength={{ x: 0.08, y: 0.08 }}>
      <Link href={`/${basePath}/${article.id}/${article.title}`}>
        <div className="group flex flex-col h-40 w-lg rounded-3xl relative hover:shadow-2xl p-5 transition-all duration-300 transform-gpu hover:scale-105 hover:bg-slate-200 dark:hover:bg-slate-800">
          <div className="flex-grow">
            <h1 className="text-2xl font-bold">{article.title}</h1>
          </div>
          <hr className="w-0 group-hover:w-full transition-all duration-300"></hr>
          <div className="">
            {article.tags.map((tag) => (
              <span key={tag} className="px-1">{`#${tag}`}</span>
            ))}
          </div>
        </div>
      </Link>
    </MagneticElement>
  );
}
