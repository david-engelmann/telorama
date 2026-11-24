import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "telorama · verdict viewer",
  description:
    "One recorded run of the telorama investigation layer over committed fixture data, rendered exactly as the tools produced it.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
