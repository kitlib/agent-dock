[Root](../CLAUDE.md) > **src**

# Frontend Module (src)

## Responsibilities

The frontend renders the desktop experience for browsing agents and local resources, selecting workspace context, and presenting detailed views for skills, MCP entries, and subagents.

## Entry Points

- **App bootstrap**: `main.tsx`
- **Page selection**: `main.tsx` maps pathname values to lazily loaded page components
- **Pages**: `pages/home.tsx`, `pages/about.tsx`, `pages/settings.tsx`
- **Styling entry**: `index.css`
- **I18n bootstrap**: `i18n/index.ts`

## Key Feature Areas

| Area | Path | Responsibility |
| --- | --- | --- |
| Agent APIs and types | `features/agents/` | Tauri invoke wrappers, scan target models, resolved agent/resource types |
| Home workspace | `features/home/` | Workspace state, selection, filtering, loading local skills, resource composition |
| Resource catalog | `features/resources/` | Local resource definitions, discovery shaping, details presentation |
| Marketplace mocks | `features/marketplace/` | Mock marketplace items and install state scaffolding |
| Shared UI | `components/` | Page-level and shared React UI; `components/ui/` is shadcn-generated |

## Current Frontend Focus

1. Resolve managed agents into workspace rails and selection state.
2. Convert managed agent roots into skill scan targets by appending `/skills`.
3. Fetch local skill summaries and details through Tauri APIs.
4. Merge local skills with cataloged MCP/subagent resources in the home workspace.

## Important Files

- `main.tsx` — top-level pathname-to-page mapping
- `features/home/use-home-workspace.ts` — central workspace state and local skill loading flow
- `features/agents/api.ts` — Tauri invoke wrappers for agents and skills
- `features/agents/types.ts` — shared frontend data contracts
- `features/resources/core/resource-catalog.ts` — built-in local MCP/subagent resource definitions
- `features/resources/core/components/resource-detail.tsx` — resource detail rendering

## Data Flow

1. Home workspace loads resolved agents.
2. Managed, visible agents become skill scan targets.
3. `features/agents/api.ts` calls `list_local_skills` / `get_local_skill_detail`.
4. Returned skills are normalized into `SkillResource` records.
5. Resource discovery utilities merge local skills with other resource kinds for browse/add flows.

## Development Commands

```bash
pnpm dev
pnpm tauri dev
pnpm format
pnpm format:check
pnpm lint
pnpm check
```

## Frontend Conventions

- Use `@/` imports for all internal frontend modules.
- Keep page orchestration in `pages/` and feature logic in `features/`.
- Prefer colocating feature hooks, typed models, and presentation helpers within the feature directory.
- Do not edit generated wrappers under `components/ui/`; change consuming code instead.
- Keep search/filter/sort logic in feature utilities or hooks, not inline in page JSX.

## Extension Guidelines

### Add a new Tauri-backed frontend capability

1. Add or update the corresponding invoke wrapper in `features/agents/api.ts` or a domain-specific API module.
2. Extend shared types in the relevant `features/**/types.ts` file.
3. Load data inside a feature hook rather than directly in a page component.
4. Keep transformation logic near the consumer hook.

### Add a new page

1. Create `pages/<name>.tsx`.
2. Add a lazy import in `main.tsx`.
3. Register the pathname in `pageMap`.
4. Ensure page-level providers like toaster/theme remain wired correctly.

### Add a new resource kind

1. Extend the `ResourceKind` union and related view models.
2. Update `resourcesByKind` and discovery utilities.
3. Add kind-specific rendering in the resource detail and list UI.
4. Keep label/search/sort behavior aligned with existing resource kinds.

## Testing and Verification

There are currently formatting, lint, and build checks exposed from the root package scripts. Prefer `pnpm check` before claiming frontend work complete.

## Boundaries

- Frontend state should not infer backend file layout beyond explicit API contracts.
- Discovery logic that touches the file system belongs in Rust scanners/services, not in React hooks.
- Hardcoded mock resources are acceptable only for marketplace/local catalog placeholders, not for managed agent or local skill data.
