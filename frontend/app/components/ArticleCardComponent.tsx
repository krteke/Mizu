import { ArticleCard } from "@/types/types";
import Link from "next/link";

export default function ArticleCardComponent({
  article,
  basePath,
}: {
  article: ArticleCard;
  basePath: string;
}) {
  return (
    <div className="flex h-48 w-xl rounded-3xl flex-col relative p-2 bg-amber-50">
      <Link
        href={`/${basePath}/${article.id}/${article.title}`}
        className="w-full h-full absolute"
      />
      <h1 className="text-2xl font-bold">{article.title}</h1>
      <div className="text-2xl">
        {article.tags.map((tag) => (
          <span key={tag}>{tag}</span>
        ))}
      </div>
    </div>
  );
}
