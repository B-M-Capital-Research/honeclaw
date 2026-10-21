// Credential-free pure translation excerpt of the deployed proxy, 2026-10-21.
// No server startup, OAuth/client secrets, config, or account data.
const SIG_SEPARATOR = "__sig__";

/// Gemini 3 returns a per-call `thoughtSignature` beside the `functionCall`,
/// and requires it back verbatim on the next turn or the follow-up 400s. An
/// OpenAI-shaped client drops unknown fields, so both the upstream call id and
/// the signature ride inside the one field that does survive: the call id.
function encodeToolCallId(upstreamId, signature) {
  const id = upstreamId || `call_${crypto.randomUUID()}`;
  if (!signature) return id;
  return `${id}${SIG_SEPARATOR}${Buffer.from(signature, "utf8").toString("base64url")}`;
}

function decodeToolCallId(id) {
  if (typeof id !== "string") return {};
  if (!id.includes(SIG_SEPARATOR)) return { upstreamId: id };
  const [upstreamId, encoded] = id.split(SIG_SEPARATOR);
  try {
    return { upstreamId, signature: Buffer.from(encoded, "base64url").toString("utf8") };
  } catch {
    return { upstreamId };
  }
}

function contentToText(content) {
  if (typeof content === "string") return content;
  if (Array.isArray(content)) {
    return content.map((p) => (typeof p === "string" ? p : p?.text || "")).join("");
  }
  return content == null ? "" : String(content);
}

/// Gemini reads images and PDFs natively, so OpenAI media parts become
/// inlineData rather than being flattened away. Only data: URIs are accepted —
/// a remote URL would make the proxy fetch arbitrary hosts on the caller's
/// behalf, and Code Assist has no fileData story for this OAuth path.
const MAX_INLINE_BYTES = 15 * 1024 * 1024;

function mediaPartsFrom(content) {
  if (!Array.isArray(content)) return [];
  const parts = [];
  for (const part of content) {
    if (!part || typeof part !== "object") continue;
    const url =
      part.type === "image_url" ? part.image_url?.url :
      part.type === "input_image" ? part.image_url?.url ?? part.url :
      part.type === "input_file" ? part.file_data ?? part.file?.file_data :
      undefined;
    if (typeof url !== "string") continue;
    const match = /^data:([^;,]+);base64,(.*)$/s.exec(url);
    if (!match) {
      console.warn(`[proxy] dropping non-data media part (${String(url).slice(0, 40)}…)`);
      continue;
    }
    const [, mimeType, data] = match;
    const bytes = Math.floor((data.length * 3) / 4);
    if (bytes > MAX_INLINE_BYTES) {
      console.warn(`[proxy] dropping oversized media part: ${bytes}B > ${MAX_INLINE_BYTES}B`);
      continue;
    }
    parts.push({ inlineData: { mimeType, data } });
  }
  return parts;
}

/// Gemini takes an OpenAPI subset. Unknown JSON Schema keywords make the whole
/// request 400, so project the schema down. Scope this to tool PARAMETERS only:
/// running it over conversation history rewrites argument values that happen to
/// use keys like `title` or `format`.
function sanitizeSchema(node, defs = {}) {
  if (Array.isArray(node)) return node.map((n) => sanitizeSchema(n, defs));
  if (!node || typeof node !== "object") return node;

  if (node.$ref) {
    const name = String(node.$ref).split("/").pop();
    return defs[name] ? sanitizeSchema(defs[name], defs) : { type: "string" };
  }
  const localDefs = { ...defs, ...(node.$defs || node.definitions || {}) };
  if (Array.isArray(node.allOf)) {
    return node.allOf.reduce(
      (acc, branch) => {
        const flat = sanitizeSchema(branch, localDefs);
        return {
          ...acc,
          ...flat,
          properties: { ...(acc.properties || {}), ...(flat.properties || {}) },
          required: [...(acc.required || []), ...(flat.required || [])],
        };
      },
      { type: "object" },
    );
  }
  const union = node.anyOf || node.oneOf;
  if (Array.isArray(union)) {
    const nonNull = union.filter((b) => b && b.type !== "null");
    const picked = nonNull.reduce(
      (best, b) => (Object.keys(b || {}).length > Object.keys(best || {}).length ? b : best),
      nonNull[0] || { type: "string" },
    );
    const out = sanitizeSchema(picked, localDefs);
    if (nonNull.length !== union.length) out.nullable = true;
    return out;
  }

  const ALLOWED = new Set([
    "type", "format", "description", "nullable", "enum", "items",
    "properties", "required", "minimum", "maximum", "example",
  ]);
  const out = {};
  for (const [key, value] of Object.entries(node)) {
    if (!ALLOWED.has(key)) continue;
    if (key === "properties") {
      out.properties = Object.fromEntries(
        Object.entries(value || {}).map(([k, v]) => [k, sanitizeSchema(v, localDefs)]),
      );
    } else if (key === "items") {
      out.items = sanitizeSchema(value, localDefs);
    } else if (key === "enum" && Array.isArray(value)) {
      out.enum = value.map((v) => String(v));
    } else {
      out[key] = value;
    }
  }
  if (out.enum && !out.type) out.type = "string";
  return out;
}

function stableSessionId(chat) {
  const seed =
    (chat.messages || []).find((m) => m.role === "user")?.content ?? "hone";
  const digest = crypto.createHash("sha256").update(contentToText(seed)).digest();
  return `-${digest.readBigUInt64BE(0) % (2n ** 62n)}`;
}

function chatToAntigravityRequest(chat) {
  const systemParts = [];
  const contents = [];
  const push = (role, part) => {
    const last = contents[contents.length - 1];
    if (last && last.role === role) last.parts.push(part);
    else contents.push({ role, parts: [part] });
  };

  for (const msg of chat.messages || []) {
    if (msg.role === "system" || msg.role === "developer") {
      const text = contentToText(msg.content);
      if (text) systemParts.push({ text });
      continue;
    }
    if (msg.role === "tool") {
      const text = contentToText(msg.content);
      let response;
      try {
        const parsed = JSON.parse(text);
        response = parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed : { result: parsed };
      } catch {
        response = { result: text };
      }
      const { upstreamId } = decodeToolCallId(msg.tool_call_id);
      const functionResponse = { name: msg.name || "tool", response };
      if (upstreamId) functionResponse.id = upstreamId;
      push("user", { functionResponse });
      continue;
    }
    if (msg.role === "assistant") {
      // `reasoning_content` is hone's replay field, not a Gemini one — drop the
      // text but keep any signature it carried, re-attached to its call below.
      const text = contentToText(msg.content);
      if (text) push("model", { text });
      for (const call of msg.tool_calls || []) {
        let args = {};
        try {
          args = JSON.parse(call.function?.arguments || "{}");
        } catch {
          args = {};
        }
        const { upstreamId, signature } = decodeToolCallId(call.id);
        const functionCall = { name: call.function?.name, args };
        if (upstreamId) functionCall.id = upstreamId;
        const part = { functionCall };
        if (signature) part.thoughtSignature = signature;
        push("model", part);
      }
      continue;
    }
    const text = contentToText(msg.content);
    if (text) push("user", { text });
    for (const media of mediaPartsFrom(msg.content)) push("user", media);
  }

  const request = { sessionId: stableSessionId(chat), contents };
  if (systemParts.length) request.systemInstruction = { role: "user", parts: systemParts };

  const functions = (chat.tools || [])
    .filter((t) => t?.type === "function" && t.function)
    .map((t) => ({
      name: t.function.name,
      description: t.function.description || "",
      parameters: sanitizeSchema(t.function.parameters || { type: "object", properties: {} }),
    }));
  if (functions.length) {
    request.tools = [{ functionDeclarations: functions }];
    const tc = chat.tool_choice;
    if (tc === "none") request.toolConfig = { functionCallingConfig: { mode: "NONE" } };
    else if (tc === "required") request.toolConfig = { functionCallingConfig: { mode: "ANY" } };
    else if (tc && tc.type === "function" && tc.function?.name) {
      request.toolConfig = {
        functionCallingConfig: { mode: "ANY", allowedFunctionNames: [tc.function.name] },
      };
    } else request.toolConfig = { functionCallingConfig: { mode: "AUTO" } };
  }

  // temperature/top_p/top_k were deprecated on Gemini 3.x and are rejected;
  // max_tokens is likewise not accepted on this path. Only thinking level rides.
  // 3.7 always thinks, but it only STREAMS thought summaries when asked to;
  // without includeThoughts the thinking track downstream stays empty even
  // though thinking tokens are billed.
  const effort = chat.reasoning?.effort ?? chat.reasoning_effort;
  const thinkingConfig = { includeThoughts: true };
  if (effort) thinkingConfig.thinkingLevel = effort === "minimal" ? "low" : effort;
  request.generationConfig = { thinkingConfig };
  if (chat.response_format?.type === "json_object") {
    request.generationConfig = {
      ...(request.generationConfig || {}),
      responseMimeType: "application/json",
    };
  }
  return request;
}

const FINISH_MAP = {};
