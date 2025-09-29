export default async function Page({
  params,
}: {
  params: Promise<{ articleId: string; articleTitle: string }>;
}) {
  const { articleId, articleTitle } = await params;
  return (
    <div className="flex w-full min-h-dvh pt-[var(--header-h)]">
      {articleId} {articleTitle}
    </div>
  );
}
