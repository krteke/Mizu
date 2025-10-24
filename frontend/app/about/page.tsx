import { ArticleCard } from "@/types/types";
import ArticleList from "../components/ArticleList";

export default function About() {
  const articles: ArticleCard[] = [
    { id: "1", content: "jsodf1", tags: ["tag1", "tag2"], title: "1" },
    { id: "2", content: "jsodf2", tags: ["tag1", "tag2"], title: "2" },
    { id: "3", content: "jsodf3", tags: ["tag1", "tag2"], title: "3" },
    { id: "4", content: "jsodf4", tags: ["tag1", "tag2"], title: "4" },
    { id: "5", content: "jsodf5", tags: ["tag1", "tag2"], title: "5" },
    { id: "6", content: "jsodf6", tags: ["tag1", "tag2"], title: "6" },
    { id: "7", content: "jsodf7", tags: ["tag1", "tag2"], title: "7" },
    { id: "8", content: "jsodf8", tags: ["tag1", "tag2"], title: "8" },
    { id: "9", content: "jsodf9", tags: ["tag1", "tag2"], title: "9" },
  ];

  return (
    // <div className="flex">
    // <div className="pt-[var(--header-h)] min-h-[calc(100dvh-var(--footer-h))]">
    // </div>
    // </div>
    <div className="flex flex-row w-full min-h-[calc(100vh-var(--footer-h))] pt-[var(--header-h)]">
      <div className="flex-1"></div>
      <div className="flex items-center justify-center flex-1">
        {articles.length > 0 ? (
          <ArticleList category="article" cards={articles} />
        ) : (
          <div></div>
        )}
      </div>
      <div className="flex-1"></div>
    </div>
  );
}
