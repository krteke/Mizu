export default async function Page({
  params,
}: {
  params: Promise<{ noteId: string; noteTitle: string }>;
}) {
  const { noteId, noteTitle } = await params;
  return (
    <div className="flex w-full min-h-dvh pt-[var(--header-h)]">
      {noteId} {noteTitle}
    </div>
  );
}
