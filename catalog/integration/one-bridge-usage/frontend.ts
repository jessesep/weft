import type { NodeDef } from "$lib/node-types";

export const node: NodeDef = {
  type: "OneBridgeUsage",
  label: "ONE Bridge Usage",
  category: "integration",
  inputs: [
    { name: "agent", type: "String", required: false, description: "Filter to one agent (optional)" },
    { name: "days", type: "Number", required: false, description: "Number of days to look back (default 1)" },
  ],
  outputs: [
    { name: "total_cost", type: "Number", description: "Total cost in USD across all agents" },
    { name: "by_day", type: "List", description: "Cost breakdown by day" },
    { name: "by_agent", type: "List", description: "Cost breakdown by agent" },
  ],
  fields: [
    { name: "bridge_url", type: "text", label: "Bridge URL", default: "http://localhost:8070" },
  ],
};
