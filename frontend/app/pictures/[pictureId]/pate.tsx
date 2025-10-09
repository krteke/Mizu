export default async function Page({
  params,
}: {
  params: Promise<{ pictureId: string; pictureTitle: string }>;
}) {
  const { pictureId, pictureTitle } = await params;
  return (
    <div className="flex w-full min-h-dvh pt-[var(--header-h)]">
      {pictureId} {pictureTitle}
    </div>
  );
}
