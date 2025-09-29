export default async function Page({
  params,
}: {
  params: Promise<{ talkId: string; talkTitle: string }>;
}) {
  const { talkId, talkTitle } = await params;
  return (
    <div className="flex w-full min-h-dvh pt-[var(--header-h)]">
      {talkId} {talkTitle}
    </div>
  );
}
