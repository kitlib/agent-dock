# LobeHub Icons

## Install

```bash
pnpm add @lobehub/icons
```

## Direct icons

```tsx
import { OpenAI, Claude } from "@lobehub/icons";

<OpenAI size={20} />
<Claude size={20} />
```

## Dynamic provider/model icons

```tsx
import { ModelIcon, ProviderIcon } from "@lobehub/icons";

<ProviderIcon provider="openai" size={20} />
<ModelIcon model="claude-3-opus" size={20} />
```

## Notes

- Not every icon supports every variant.
- Prefer direct icons, `ProviderIcon`, and `ModelIcon` first.
- Only check advanced variants when needed.
