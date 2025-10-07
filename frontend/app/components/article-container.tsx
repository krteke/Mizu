import { ArticleDigital } from "@/types/types";

export default function ArticleContainer({
  article,
}: {
  article: ArticleDigital;
}) {
  return (
    <div className="flex">
      <h1>{article.title}</h1>
      <p>{article.summary}</p>
      <p>{article.content}</p>
    </div>
  );
}
