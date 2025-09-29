export default async function Page({
  params,
}: {
  params: Promise<{ thinkId: string; thinkTitle: string }>;
}) {
  const { thinkId, thinkTitle } = await params;
  return (
    <div className="flex w-full min-h-dvh pt-[var(--header-h)]">
      {thinkId} {thinkTitle}
    </div>
  );
}
