import { promises as fs } from "fs";
import path from "path";
import Viewer, { type RecordData } from "./Viewer";

export default async function Page() {
  const raw = await fs.readFile(
    path.join(process.cwd(), "data", "record.json"),
    "utf-8"
  );
  const data: RecordData = JSON.parse(raw);
  return <Viewer data={data} />;
}
