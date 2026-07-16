"use server";

import * as k8s from "@kubernetes/client-node";
import * as fs from "fs/promises";
import * as path from "path";
import { revalidatePath } from "next/cache";

const NAMESPACE = process.env.K8S_NAMESPACE || "starbunk";
const LOCAL_CONFIG_PATH = path.join(process.cwd(), "..", "config");

function isNotFound(err: unknown): boolean {
  return (
    typeof err === "object" &&
    err !== null &&
    ("code" in err || "statusCode" in err) &&
    ((err as Record<string, unknown>).code === 404 ||
      (err as Record<string, unknown>).statusCode === 404)
  );
}

function errMsg(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

let k8sAppsApi: k8s.AppsV1Api | null = null;
let k8sCoreApi: k8s.CoreV1Api | null = null;

try {
  const kc = new k8s.KubeConfig();
  kc.loadFromDefault();
  k8sAppsApi = kc.makeApiClient(k8s.AppsV1Api);
  k8sCoreApi = kc.makeApiClient(k8s.CoreV1Api);
} catch (error) {
  console.warn("Failed to initialize Kubernetes client. Continuing in local mode.", error);
}

export interface BotDeploymentStatus {
  name: string;
  replicas: number;
  readyReplicas: number;
  availableReplicas: number;
  status: "Running" | "Stopped" | "Degraded" | "Unknown";
}

export async function getBotDeployments(): Promise<BotDeploymentStatus[]> {
  const bots = ["bunkbot", "covabot", "djcova", "bluebot"];

  if (!k8sAppsApi) {
    return bots.map(bot => ({
      name: bot,
      replicas: 1,
      readyReplicas: 1,
      availableReplicas: 1,
      status: "Running",
    }));
  }

  try {
    const statuses = await Promise.all(bots.map(async (bot) => {
      try {
        const dep = await k8sAppsApi!.readNamespacedDeployment({ name: bot, namespace: NAMESPACE });
        const replicas = dep.spec?.replicas ?? 0;
        const ready = dep.status?.readyReplicas ?? 0;

        let status: BotDeploymentStatus["status"] = "Unknown";
        if (replicas === 0) status = "Stopped";
        else if (ready === replicas) status = "Running";
        else status = "Degraded";

        return {
          name: bot,
          replicas,
          readyReplicas: ready,
          availableReplicas: dep.status?.availableReplicas ?? 0,
          status,
        };
      } catch (err: unknown) {
        if (isNotFound(err)) {
          return { name: bot, replicas: 0, readyReplicas: 0, availableReplicas: 0, status: "Unknown" as const };
        }
        throw err;
      }
    }));
    return statuses;
  } catch (error) {
    console.error("Error fetching deployments:", error);
    return bots.map(bot => ({ name: bot, replicas: 0, readyReplicas: 0, availableReplicas: 0, status: "Unknown" as const }));
  }
}

export async function setBotState(botName: string, action: "start" | "stop" | "restart") {
  if (!k8sAppsApi) {
    console.log(`[Mock] Action ${action} performed on ${botName}`);
    revalidatePath("/");
    return { success: true };
  }

  try {
    if (action === "start" || action === "stop") {
      const replicas = action === "start" ? 1 : 0;
      await k8sAppsApi.patchNamespacedDeployment({
        name: botName,
        namespace: NAMESPACE,
        body: [{ op: "replace", path: "/spec/replicas", value: replicas }],
      });
    } else {
      const timestamp = new Date().toISOString();
      try {
        await k8sAppsApi.patchNamespacedDeployment({
          name: botName,
          namespace: NAMESPACE,
          body: [{
            op: "replace",
            path: "/spec/template/metadata/annotations/kubectl.kubernetes.io~1restartedAt",
            value: timestamp,
          }],
        });
      } catch (e: unknown) {
        if (typeof e === "object" && e !== null && ((e as Record<string, unknown>).code === 422 || (e as Record<string, unknown>).statusCode === 422)) {
          await k8sAppsApi.patchNamespacedDeployment({
            name: botName,
            namespace: NAMESPACE,
            body: [{
              op: "add",
              path: "/spec/template/metadata/annotations",
              value: { "kubectl.kubernetes.io/restartedAt": timestamp },
            }],
          });
        } else {
          throw e;
        }
      }
    }

    revalidatePath("/");
    return { success: true };
  } catch (error: unknown) {
    console.error(`Error performing ${action} on ${botName}:`, error);
    return { success: false, error: errMsg(error) };
  }
}

export async function getBotConfigs(botName: "bunkbot" | "covabot"): Promise<Record<string, string>> {
  if (!k8sCoreApi) {
    try {
      const dirPath = path.join(LOCAL_CONFIG_PATH, botName);
      await fs.mkdir(dirPath, { recursive: true });
      const files = await fs.readdir(dirPath);
      const configs: Record<string, string> = {};
      for (const file of files) {
        if (file.endsWith(".yml") || file.endsWith(".yaml")) {
          configs[file] = await fs.readFile(path.join(dirPath, file), "utf-8");
        }
      }
      return configs;
    } catch (e) {
      console.error(`Failed to read local configs for ${botName}`, e);
      return {};
    }
  }

  try {
    const cm = await k8sCoreApi.readNamespacedConfigMap({ name: `${botName}-configs`, namespace: NAMESPACE });
    return cm.data ?? {};
  } catch (error: unknown) {
    if (isNotFound(error)) return {};
    console.error(`Error reading configmap for ${botName}:`, error);
    throw error;
  }
}

export async function updateBotConfig(
  botName: "bunkbot" | "covabot",
  filename: string,
  content: string | null,
): Promise<{ success: boolean; error?: string }> {
  if (!k8sCoreApi) {
    try {
      const filePath = path.join(LOCAL_CONFIG_PATH, botName, filename);
      if (content === null) {
        await fs.unlink(filePath).catch(() => {});
      } else {
        await fs.mkdir(path.dirname(filePath), { recursive: true });
        await fs.writeFile(filePath, content, "utf-8");
      }
      revalidatePath(`/${botName}`);
      return { success: true };
    } catch (e: unknown) {
      return { success: false, error: errMsg(e) };
    }
  }

  try {
    const configMapName = `${botName}-configs`;
    let currentData: Record<string, string> = {};
    try {
      const cm = await k8sCoreApi.readNamespacedConfigMap({ name: configMapName, namespace: NAMESPACE });
      currentData = cm.data ?? {};
    } catch (e: unknown) {
      if (!isNotFound(e)) throw e;
    }

    if (content === null) {
      delete currentData[filename];
    } else {
      currentData[filename] = content;
    }

    await k8sCoreApi.replaceNamespacedConfigMap({
      name: configMapName,
      namespace: NAMESPACE,
      body: {
        apiVersion: "v1",
        kind: "ConfigMap",
        metadata: { name: configMapName, namespace: NAMESPACE },
        data: currentData,
      },
    });

    revalidatePath(`/${botName}`);
    return { success: true };
  } catch (error: unknown) {
    console.error(`Error writing configmap for ${botName}:`, error);
    return { success: false, error: errMsg(error) };
  }
}
