import type { NodeDef } from "$lib/node-types";

export const node: NodeDef = {
  type: "OneTelethonAdmin",
  label: "ONE Telethon Admin",
  category: "integration",
  inputs: [
    { name: "operation", type: "String", required: true, description: "Telethon op name (e.g. send_message, create_topic, promote_admin)" },
    { name: "payload", type: "JsonDict", required: false, description: "Operation-specific arguments" },
  ],
  outputs: [
    { name: "result", type: "JsonDict", description: "Full response from telethon-service" },
    { name: "ok", type: "Boolean", description: "True if operation succeeded" },
  ],
  fields: [
    { name: "telethon_url", type: "text", label: "Telethon Service URL", default: "http://127.0.0.1:13100" },
  ],
};
