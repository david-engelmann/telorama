"use client";

import { useState } from "react";

export interface EvidencePoint {
  label: string;
  value: string;
}

export interface Finding {
  rank: number;
  kind: string;
  subject: string;
  statement: string;
  evidence: EvidencePoint[];
  exemplars: string[];
}

export interface Anchor {
  rank: number;
  panel: number;
}

export interface IncidentData {
  svg: string;
  svg_fnv1a64: string;
  investigation: Finding[];
  anchors: Anchor[];
  summary: {
    title: string;
    headline: string;
    generated_by: string;
    question: string;
    from_unix_ms: number;
    to_unix_ms: number;
    panels: string[];
  };
}

export interface FrameData {
  frame: number;
  turns: number;
  findings: Finding[];
  anchors: Anchor[];
  panels: string[];
  svg: string;
  svg_fnv1a64: string;
}

export interface RecordData {
  record: {
    generator: string;
    engine_version: string;
    scene_version: string;
    fixtures: string[];
  };
  incident: IncidentData;
  session: {
    session_id: string;
    dialect: string;
    frames: FrameData[];
  };
}

const KIND_LABELS: Record<string, string> = {
  errors: "Errors",
  latency: "Latency",
  wait: "Wait",
  cost: "Cost",
};

const PANEL_LABELS: Record<string, string> = {
  service_map: "service map",
  red_row: "error rate row",
  waterfall: "waterfall",
  heatmap: "heatmap",
  agent_run: "agent run",
};

function panelLabel(type: string): string {
  return PANEL_LABELS[type] ?? type;
}

function anchorPanel(
  anchors: Anchor[],
  panels: string[],
  rank: number
): string | null {
  const anchor = anchors.find((a) => a.rank === rank);
  if (!anchor) return null;
  const type = panels[anchor.panel];
  return type ? panelLabel(type) : null;
}

function Inspector({
  finding,
  shownIn,
}: {
  finding: Finding;
  shownIn: string | null;
}) {
  return (
    <div className="inspector">
      <div className="inspector-head">
        <span className={`kind-dot kind-${finding.kind}`} />
        <span className="inspector-kind">
          {KIND_LABELS[finding.kind] ?? finding.kind} · rank {finding.rank}
        </span>
      </div>
      <div className="inspector-subject">{finding.subject}</div>
      <p className="inspector-statement">{finding.statement}</p>
      <dl className="inspector-facts">
        {finding.evidence.map((point) => (
          <div className="fact" key={point.label}>
            <dt>{point.label}</dt>
            <dd>{point.value}</dd>
          </div>
        ))}
        {finding.exemplars.length > 0 && (
          <div className="fact">
            <dt>exemplars</dt>
            <dd className="mono">{finding.exemplars.join(" · ")}</dd>
          </div>
        )}
        {shownIn && (
          <div className="fact">
            <dt>shown in</dt>
            <dd>the {shownIn} panel</dd>
          </div>
        )}
      </dl>
    </div>
  );
}

function FindingsPane({
  findings,
  anchors,
  panels,
}: {
  findings: Finding[];
  anchors: Anchor[];
  panels: string[];
}) {
  const [selected, setSelected] = useState<number>(findings[0]?.rank ?? 0);
  const current =
    findings.find((f) => f.rank === selected) ?? findings[0];
  if (!current) return null;
  return (
    <div className="findings-pane">
      <Inspector
        finding={current}
        shownIn={anchorPanel(anchors, panels, current.rank)}
      />
      <ul className="findings-list">
        {findings.map((finding) => (
          <li key={finding.rank}>
            <button
              type="button"
              className={
                finding.rank === current.rank ? "row selected" : "row"
              }
              onClick={() => setSelected(finding.rank)}
            >
              <span className={`kind-dot kind-${finding.kind}`} />
              <span className="row-kind">
                {KIND_LABELS[finding.kind] ?? finding.kind}
              </span>
              <span className="row-subject">{finding.subject}</span>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}

function SceneSvg({ svg, label }: { svg: string; label: string }) {
  return (
    <div
      className="scene"
      role="img"
      aria-label={label}
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  );
}

export default function Viewer({ data }: { data: RecordData }) {
  const { incident, session } = data;
  const [frameIdx, setFrameIdx] = useState<number>(session.frames.length - 1);
  const frame = session.frames[frameIdx];
  const kindCounts = new Map<string, number>();
  for (const finding of incident.investigation) {
    kindCounts.set(finding.kind, (kindCounts.get(finding.kind) ?? 0) + 1);
  }
  const finalFrame = session.frames[session.frames.length - 1];
  if (finalFrame) {
    for (const finding of finalFrame.findings) {
      kindCounts.set(finding.kind, (kindCounts.get(finding.kind) ?? 0) + 1);
    }
  }
  const canonical = ["errors", "latency", "wait", "cost"];
  const kindOrder = [
    ...canonical.filter((kind) => kindCounts.has(kind)),
    ...[...kindCounts.keys()].filter((kind) => !canonical.includes(kind)),
  ];

  return (
    <main>
      <header className="masthead">
        <div className="wordmark">TELORAMA · verdict viewer</div>
        <p className="tagline">
          One recorded run of the investigation layer over real fixture
          data, rendered exactly as the tools produced it. The page
          recomputes no finding, figure, or hash.
        </p>
        <p className="provenance">
          Record generated by {data.record.generator} · engine{" "}
          {data.record.engine_version} · scene spec{" "}
          {data.record.scene_version} · fixtures{" "}
          {data.record.fixtures.join(", ")}
        </p>
      </header>

      <section className="summary-bar" aria-label="Record summary">
        <div className="summary-lead">
          <span className="summary-label">Look here first</span>
          <span className="summary-headline">{incident.summary.headline}</span>
        </div>
        <div className="summary-counts">
          {kindOrder.map((kind) => (
            <div className="count" key={kind}>
              <span className={`kind-dot kind-${kind}`} />
              <span className="count-value">{kindCounts.get(kind)}</span>
              <span className="count-label">
                {KIND_LABELS[kind] ?? kind}
              </span>
            </div>
          ))}
          <span className="summary-scope">
            Incident and the session's final frame
          </span>
        </div>
      </section>

      <section className="record-section">
        <h2>{incident.summary.title}</h2>
        <p className="section-question">{incident.summary.question}</p>
        <div className="section-grid">
          <SceneSvg svg={incident.svg} label={incident.summary.title} />
          <FindingsPane
            findings={incident.investigation}
            anchors={incident.anchors}
            panels={incident.summary.panels}
          />
        </div>
        <p className="scene-hash mono">
          svg fnv1a64 {incident.svg_fnv1a64}
        </p>
      </section>

      <section className="record-section">
        <h2>Session {session.session_id}</h2>
        <p className="section-question">
          A recorded {session.dialect} session export, one frame per
          turn prefix. Stepping replaces the whole frame. The data
          never animates.
        </p>
        <nav className="stepper" aria-label="Frame stepper">
          <button
            type="button"
            disabled={frameIdx === 0}
            onClick={() => setFrameIdx(frameIdx - 1)}
          >
            Previous
          </button>
          <span className="stepper-label">
            Frame {frame.frame} of {session.frames.length} · after turn{" "}
            {frame.turns}
          </span>
          <button
            type="button"
            disabled={frameIdx === session.frames.length - 1}
            onClick={() => setFrameIdx(frameIdx + 1)}
          >
            Next
          </button>
        </nav>
        <div className="section-grid">
          <SceneSvg
            svg={frame.svg}
            label={`Session frame ${frame.frame}`}
          />
          <FindingsPane
            key={frame.frame}
            findings={frame.findings}
            anchors={frame.anchors}
            panels={frame.panels}
          />
        </div>
        <p className="scene-hash mono">svg fnv1a64 {frame.svg_fnv1a64}</p>
      </section>

      <footer className="footer">
        <p>
          This page proves one narrow thing. The record it renders
          matches what the tools produced from the committed fixtures,
          byte for byte, under a freshness gate in the test suite. It
          does not prove these are the right findings for your data,
          and the page recomputes no finding, figure, or hash.
        </p>
        <p>
          The corpus is the OpenTelemetry demo checkout trace and a
          recorded Claude Code session export, both committed as test
          fixtures. Between them they exercise every finding kind the
          investigation layer emits. The incident scene's heatmap
          panel carries no finding, so the quiet case is on the page
          too.
        </p>
      </footer>
    </main>
  );
}
