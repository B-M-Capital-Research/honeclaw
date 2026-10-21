import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import vm from "node:vm";

function loadTranslation(path) {
  const source = fs.readFileSync(path, "utf8");
  const start = source.indexOf("const SIG_SEPARATOR = ");
  const end = source.indexOf("const FINISH_MAP = ", start);
  assert.ok(start >= 0 && end > start, "expected pure proxy translation section");
  // Evaluate only pure conversion helpers. Never execute server/OAuth setup,
  // even when checking a complete private deployment source copy.
  return vm.runInNewContext(
    source.slice(start, end) + "\n({ chatToAntigravityRequest, encodeToolCallId, decodeToolCallId })",
    { Buffer, crypto },
    { timeout: 1000 },
  );
}

const original = loadTranslation(process.argv[2]);
const patched = loadTranslation(process.argv[3]);
const call = (id, name) => ({ id, type: "function", function: { name, arguments: '{"q":"example"}' } });
const assistant = (...calls) => ({ role: "assistant", content: null, tool_calls: calls });
const result = (id, name) => ({ role: "tool", tool_call_id: id, content: '{"ok":true}', ...(name ? { name } : {}) });
const chat = (...messages) => ({ messages: [{ role: "user", content: "research test" }, ...messages] });
const convert = (...messages) => patched.chatToAntigravityRequest(chat(...messages));
const parts = (request) => request.contents.flatMap((content) => content.parts);
const responses = (request) => parts(request).filter((part) => part.functionResponse).map((part) => part.functionResponse);
const names = (request) => Array.from(responses(request), (response) => response.name);

// This is the actual OpenCode contract: tool results have no name field.
const standard = chat(assistant(call("call_1", "todowrite")), result("call_1"));
assert.equal(responses(original.chatToAntigravityRequest(standard))[0].name, "tool", "fixture must reproduce the original defect");
assert.equal(responses(patched.chatToAntigravityRequest(standard))[0].name, "todowrite");

// Several tools can complete in a different order from their declarations.
assert.deepEqual(names(convert(
  assistant(call("parallel_a", "web_search"), call("parallel_b", "data_fetch")),
  result("parallel_b"), result("parallel_a"),
)), ["data_fetch", "web_search"]);

// Repeated tools and distinct tools across rounds resolve by id, not position.
assert.deepEqual(names(convert(
  assistant(call("round1", "web_search")), result("round1"),
  assistant(call("round2", "web_search")), result("round2"),
  assistant(call("round3", "skill_tool")), result("round3"),
)), ["web_search", "web_search", "skill_tool"]);

// Keep signature transport byte-for-byte unchanged; match the full public id,
// even when two ids happen to decode to the same upstream id.
const signature = "opaque+/signature==:汉字";
const signed = patched.encodeToolCallId("upstream_1", signature);
assert.equal(signed, original.encodeToolCallId("upstream_1", signature));
const otherSigned = patched.encodeToolCallId("upstream_1", "other-signature");
const signedRequest = convert(
  assistant(call(signed, "web_search"), call(otherSigned, "data_fetch")),
  result(signed), result(otherSigned),
);
assert.deepEqual(names(signedRequest), ["web_search", "data_fetch"]);
const functionParts = parts(signedRequest).filter((part) => part.functionCall);
assert.equal(functionParts[0].thoughtSignature, signature);
assert.equal(functionParts[0].functionCall.id, "upstream_1");
assert.equal(responses(signedRequest)[0].id, "upstream_1");
assert.equal(patched.decodeToolCallId(signed).signature, signature);

// Existing explicit-name callers remain supported without assistant history;
// a matched assistant call is authoritative if a supplied name is stale.
assert.deepEqual(names(convert(result("legacy", "legacy_function"))), ["legacy_function"]);
assert.deepEqual(names(convert(
  assistant(call("known", "todowrite")), result("known", "stale_function"),
)), ["todowrite"]);

// Unknown ids cannot borrow a neighboring call's name or silently become tool.
assert.throws(() => convert(assistant(call("known", "web_search")), result("unknown")), /no matching function name/);
assert.throws(() => convert(result("unknown", "  ")), /no matching function name/);
assert.throws(() => convert(assistant(call(signed, "web_search")), result(otherSigned)), /no matching function name/);

console.log("PASS: Gemini tool response names, exact ids, multi-round calls and thought signatures");
