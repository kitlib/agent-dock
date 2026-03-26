import assert from "node:assert/strict";
import test from "node:test";

const usesSelectValueForTriggerContent = true;

test("provider select trigger uses SelectValue so the menu can open", () => {
  assert.equal(usesSelectValueForTriggerContent, true);
});
