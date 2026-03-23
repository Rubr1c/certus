import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import { Header } from "@/components/Header";
import { Providers } from "@/components/providers";
import { Sidebar } from "@/components/Sidebar";
import { Toaster } from "sonner";
import "./globals.css";

const inter = Inter({ subsets: ["latin"], variable: "--font-inter" });
const jetbrainsMono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-mono",
});

export const metadata: Metadata = {
  title: "Certus Dashboard",
  description: "Certus API Gateway Dashboard",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${inter.variable} ${jetbrainsMono.variable} font-sans antialiased`}
      >
        <Providers>
        <Toaster position="top-right" richColors />
        <div className="flex min-h-screen">
          <Sidebar />

          <div className="flex flex-1 flex-col">
            <Header />

            <main className="ml-64 mt-16 min-h-screen bg-background p-8">
              <div className="mx-auto max-w-7xl space-y-6">{children}</div>
            </main>
          </div>
        </div>
        </Providers>
      </body>
    </html>
  );
}
