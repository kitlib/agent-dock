import assert from "node:assert/strict";
import test from "node:test";

type SkillActionState = {
  showOpen: boolean;
  disableOpen: boolean;
};

function getSkillOpenActionState(resource: { kind: string; origin: string; skillPath?: string }) {
  const isLocalSkill = resource.kind === "skill" && resource.origin === "local";
  const hasSkillPath = typeof resource.skillPath === "string" && resource.skillPath.trim().length > 0;

  return {
    showOpen: isLocalSkill,
    disableOpen: !hasSkillPath,
  } satisfies SkillActionState;
}

test("skill open action is visible and enabled when local skill has skillPath", () => {
  assert.deepEqual(
    getSkillOpenActionState({
      kind: "skill",
      origin: "local",
      skillPath: "/workspace/.claude/skills/release-checklist",
    }),
    { showOpen: true, disableOpen: false }
  );
});

test("skill open action stays visible but disabled when skillPath is empty", () => {
  assert.deepEqual(
    getSkillOpenActionState({
      kind: "skill",
      origin: "local",
      skillPath: "",
    }),
    { showOpen: true, disableOpen: true }
  );
});

test("skill open action is hidden for non-skill resources", () => {
  assert.deepEqual(
    getSkillOpenActionState({
      kind: "mcp",
      origin: "local",
      skillPath: "/workspace/.claude/skills/release-checklist",
    }),
    { showOpen: false, disableOpen: false }
  );
});
