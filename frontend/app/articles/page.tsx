import { ArticleCard } from "@/types/types";
import ArticleList from "../components/ArticleList";

async function getArticleList(): Promise<ArticleCard[]> {
  try {
    const res = await fetch(`/api/posts?category=article`, {
      cache: "no-store",
    });

    if (!res.ok) return [];
    return res.json();
  } catch (e) {
    console.error("Failed to fetch article list: ", e);
    return [];
  }
}

export default async function Aritcles() {
  const articles = await getArticleList();

  return (
    <div className="flex w-full min-h-[calc(100vh-var(--footer-h))] pt-[var(--header-h)]">
      {articles.length > 0 ? (
        <ArticleList category="article" cards={articles} />
      ) : (
        <div></div>
      )}
    </div>
  );
}
