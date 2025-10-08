import ArticleContainer from "@/app/components/ArticleContainer";
import notFound from "@/app/not-found";
import { ArticleDigital } from "@/types/types";

async function getAritcle({
  articleId,
  articleTitle,
}: {
  articleId: string;
  articleTitle: string;
}): Promise<ArticleDigital | null> {
  try {
    const encodedTitle = encodeURIComponent(articleTitle);
    const res = await fetch(`/api/posts/article/${articleId}/${encodedTitle}`, {
      cache: "no-store",
    });

    if (!res.ok) {
      console.error("Failed to fetch article.", res.status, res.statusText);
      return null;
    }

    return res.json();
  } catch (e) {
    console.error("Failed to fetch article: ", e);
    return null;
  }
}

export default async function Page({
  params,
}: {
  params: Promise<{ articleId: string; articleTitle: string }>;
}) {
  const { articleId, articleTitle } = await params;
  const post = await getAritcle({ articleId, articleTitle });

  if (!post) {
    notFound();
  } else {
    return <ArticleContainer article={post} />;
  }
}
