import { promises as fs } from "fs";
import path from "path";
import Viewer, { type RecordData } from "./Viewer";

export default async function Page() {
  const raw = await fs.readFile(
    path.join(process.cwd(), "data", "record.json"),
    "utf-8"
  );
  const parsed = JSON.parse(raw);
  const data: RecordData = {
    ...parsed,
    session: {
      ...parsed.session,
      frames: parsed.session.frames.map((frame: Record<string, unknown>) => ({
        ...frame,
        panels: (
          (frame.scene as { panels?: { type?: string }[] } | undefined)
            ?.panels ?? []
        ).map((panel) => String(panel.type ?? "unknown")),
      })),
    },
  };
  return <Viewer data={data} />;
}
