import { NextRequest } from "next/server";

export const dynamic = "force-dynamic";

export async function GET(req: NextRequest) {
  const encoder = new TextEncoder();

  const stream = new ReadableStream({
    start(controller) {
      const sendEvent = (event: string, data: unknown) => {
        controller.enqueue(
          encoder.encode(`event: ${event}\ndata: ${JSON.stringify(data)}\n\n`)
        );
      };

      // Initial connection tick
      sendEvent("connected", { status: "ok", timestamp: new Date().toISOString() });

      // Interval ticker simulating live bot event telemetry
      const interval = setInterval(() => {
        const timestamp = new Date().toISOString();
        const bots = ["bunkbot", "covabot", "djcova", "bluebot", "ratbot"];
        const randomBot = bots[Math.floor(Math.random() * bots.length)];

        sendEvent("telemetry", {
          bot: randomBot,
          timestamp,
          messageRate: Math.floor(Math.random() * 15) + 1,
          latencyMs: Math.floor(Math.random() * 20) + 5,
        });

        // Periodic BlueBot trigger audit simulation
        if (Math.random() > 0.6) {
          sendEvent("bluebot_audit", {
            id: Date.now(),
            user: "DiscordUser" + Math.floor(Math.random() * 90 + 10),
            channel: "#general",
            matchedVariant: ["blue", "bleu", "bluu", "blau"][Math.floor(Math.random() * 4)],
            response: "Did somebody say Blu?",
            timestamp: new Date().toLocaleTimeString(),
          });
        }

        // Periodic BunkBot trigger audit simulation
        if (Math.random() > 0.5) {
          sendEvent("bunkbot_audit", {
            id: Date.now(),
            subBot: ["StarcraftBot", "WelcomeBot", "MemeBot"][Math.floor(Math.random() * 3)],
            trigger: "Keyword Match",
            user: "Gamer" + Math.floor(Math.random() * 50 + 1),
            response: "Automated BunkBot Dispatch",
            timestamp: new Date().toLocaleTimeString(),
          });
        }
      }, 3000);

      req.signal.addEventListener("abort", () => {
        clearInterval(interval);
        controller.close();
      });
    },
  });

  return new Response(stream, {
    headers: {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache, no-transform",
      Connection: "keep-alive",
    },
  });
}
