# Branching

| Branch | Purpose | Who pushes |
|---|---|---|
| `main` | What is released and what the docs site shows. Always working. | Only by merging `dev`. |
| `dev` | Integration branch. New features and fixes are merged here and tested first. | Merge feature branches into it. |
| `feature/<name>`, `fix/<name>` | One piece of work, branched from `dev`. | You. |

## Flow

```bash
git switch dev
git switch -c feature/my-change      # work, commit
git switch dev && git merge feature/my-change
# test on dev, then release it:
git switch main && git merge dev && git push
```

## What runs where

- **Docs site:** `.github/workflows/deploy-docs.yml` deploys to GitHub Pages only for pushes to
  `main` that touch `docs-site/`. Pushes to `dev` and feature branches run nothing, and the job
  refuses a manual run from any branch other than `main`.
- **Releases:** `npm run release` is run by hand from a clean `main` (see `RELEASING.md`).

## Recommended GitHub settings (cannot be set from the repository)

1. **Settings → Branches → Add rule** for `main`: require a pull request before merging, and
   block force pushes.
2. **Settings → Environments → github-pages → Deployment branches:** allow only `main`, so no other
   branch can deploy the docs even if a workflow file is changed.
