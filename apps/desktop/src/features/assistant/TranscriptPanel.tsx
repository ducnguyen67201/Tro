import { useAssistantStore } from "./assistantStore";

export function TranscriptPanel() {
  const transcript = useAssistantStore((state) => state.transcript);
  if (!transcript) return null;
  return (
    <section className="transcript" aria-live="polite">
      <span className="eyebrow">Tro</span>
      <p>{transcript}</p>
    </section>
  );
}
