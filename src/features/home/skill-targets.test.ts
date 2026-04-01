import assert from "node:assert/strict";
import test from "node:test";

const agentMeta: Record<string, { skills: string | null }> = {
  claude: { skills: "skills/" },
  "pi-mono": { skills: "agent/skills/" },
};

type Agent = {
  id: string;
  provider: string;
  rootPath: string;
  managed?: boolean;
  hidden?: boolean;
};

function trimTrailingSlash(value: string) {
  return value.replace(/\/+$/, "");
}

function trimLeadingSlash(value: string) {
  return value.replace(/^\/+/, "");
}

function buildSkillScanPath(agent: { provider: string; rootPath: string }) {
  const skillRelativePath = agentMeta[agent.provider]?.skills;
  if (!skillRelativePath) {
    return null;
  }

  const rootPath = trimTrailingSlash(agent.rootPath);
  const relativePath = trimLeadingSlash(trimTrailingSlash(skillRelativePath));
  return `${rootPath}/${relativePath}`;
}

function filterSkillsForAgent(
  skills: Array<{ id: string; ownerAgentId?: string | null }>,
  selectedAgentId: string | null
) {
  if (!selectedAgentId) {
    return [];
  }

  return skills.filter((skill) => skill.ownerAgentId === selectedAgentId);
}

function resolveSelectedAgent(
  visibleAgents: Agent[],
  stableAgents: Agent[],
  selectedAgentId: string
) {
  return (
    visibleAgents.find((agent) => agent.id === selectedAgentId) ??
    stableAgents.find((agent) => agent.id === selectedAgentId) ??
    visibleAgents[0] ??
    stableAgents[0] ??
    null
  );
}

function buildSkillScanTargets(agents: Agent[]) {
  return agents
    .filter((agent) => agent.managed && !agent.hidden)
    .map((agent) => ({ agentId: agent.id, rootPath: buildSkillScanPath(agent) }))
    .filter((target): target is { agentId: string; rootPath: string } => target.rootPath !== null);
}

function diagnoseSkillResult(
  scanTargets: Array<{ agentId: string; rootPath: string }>,
  skills: Array<{ id: string; ownerAgentId?: string | null }>
) {
  if (scanTargets.length > 0 && skills.length === 0) {
    return "scanner-empty";
  }

  if (skills.some((skill) => !skill.ownerAgentId)) {
    return "missing-owner";
  }

  return "ok";
}

const claudeAgent = {
  id: "agent-claude",
  provider: "claude",
  rootPath: ".claude",
  managed: true,
  hidden: false,
};

const piMonoAgent = {
  id: "agent-pi",
  provider: "pi-mono",
  rootPath: ".pi",
  managed: true,
  hidden: false,
};

const skills = [
  { id: "agent-claude::release-checklist", ownerAgentId: "agent-claude" },
  { id: "agent-pi::ops", ownerAgentId: "agent-pi" },
];

test("buildSkillScanPath uses provider matrix skills path", () => {
  assert.equal(buildSkillScanPath(claudeAgent), ".claude/skills");
  assert.equal(buildSkillScanPath(piMonoAgent), ".pi/agent/skills");
});

test("filterSkillsForAgent keeps only selected agent skills", () => {
  const filtered = filterSkillsForAgent(skills, "agent-claude");

  assert.deepEqual(
    filtered.map((skill) => skill.id),
    ["agent-claude::release-checklist"]
  );
});

test("resolveSelectedAgent keeps explicit selection even when rail search hides it", () => {
  const selectedAgent = resolveSelectedAgent([claudeAgent], [claudeAgent, piMonoAgent], "agent-pi");

  assert.equal(selectedAgent?.id, "agent-pi");
});

test("buildSkillScanTargets uses stable managed agents instead of rail-filtered agents", () => {
  assert.deepEqual(buildSkillScanTargets([claudeAgent, piMonoAgent]), [
    { agentId: "agent-claude", rootPath: ".claude/skills" },
    { agentId: "agent-pi", rootPath: ".pi/agent/skills" },
  ]);
});

test("diagnoseSkillResult flags empty scan result when targets exist", () => {
  const diagnosis = diagnoseSkillResult(buildSkillScanTargets([claudeAgent, piMonoAgent]), []);

  assert.equal(diagnosis, "scanner-empty");
});
