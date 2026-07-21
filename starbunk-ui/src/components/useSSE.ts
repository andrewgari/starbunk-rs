"use client";

import { useEffect, useState } from "react";

export interface TelemetryData {
  bot: string;
  timestamp: string;
  messageRate: number;
  latencyMs: number;
}

export interface AuditEvent {
  id: number;
  user: string;
  channel?: string;
  subBot?: string;
  trigger?: string;
  matchedVariant?: string;
  response: string;
  timestamp: string;
}

export function useSSE() {
  const [telemetry, setTelemetry] = useState<TelemetryData | null>(null);
  const [bluebotAudits, setBluebotAudits] = useState<AuditEvent[]>([]);
  const [bunkbotAudits, setBunkbotAudits] = useState<AuditEvent[]>([]);
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    const eventSource = new EventSource("/api/sse");

    eventSource.addEventListener("connected", () => {
      setIsConnected(true);
    });

    eventSource.addEventListener("telemetry", (e) => {
      try {
        const data: TelemetryData = JSON.parse(e.data);
        setTelemetry(data);
      } catch {
        // ignore parse error
      }
    });

    eventSource.addEventListener("bluebot_audit", (e) => {
      try {
        const audit: AuditEvent = JSON.parse(e.data);
        setBluebotAudits((prev) => [audit, ...prev].slice(0, 20));
      } catch {
        // ignore
      }
    });

    eventSource.addEventListener("bunkbot_audit", (e) => {
      try {
        const audit: AuditEvent = JSON.parse(e.data);
        setBunkbotAudits((prev) => [audit, ...prev].slice(0, 20));
      } catch {
        // ignore
      }
    });

    eventSource.onerror = () => {
      setIsConnected(false);
    };

    return () => {
      eventSource.close();
    };
  }, []);

  return {
    isConnected,
    telemetry,
    bluebotAudits,
    bunkbotAudits,
  };
}
