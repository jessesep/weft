import type { NodeDef } from "$lib/node-types";

export const node: NodeDef = {
  type: "BridgeRagQuery",
  label: "Bridge RAG Query",
  category: "integration",
  inputs: [
    { name: "query", type: "String", required: true, description: "Search query" },
    { name: "top_k", type: "Number", required: false, description: "Number of chunks to return (default 5)" },
  ],
  outputs: [
    { name: "chunks", type: "List", description: "Matching text chunks with source and score" },
    { name: "sources", type: "List", description: "Deduplicated source list" },
  ],
  fields: [
    { name: "bridge_url", type: "text", label: "Bridge URL", default: "http://localhost:8070" },
  ],
};
