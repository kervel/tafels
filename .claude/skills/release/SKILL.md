---
name: release
description: Use when shipping a new version, creating a release, deploying to production, or updating the server. Covers commit, tag, GitHub release, CI monitoring, and mlops repo update.
---

# Release

## Steps

### 1. Commit and push (main)
- Commit all staged changes on `main`, push to origin
- NEVER reuse a previous tag — always bump version

### 2. Determine next version
- Check latest: `gh release list --limit 1`
- Bump patch (e.g. v0.1.22 → v0.1.23) unless user specifies otherwise

### 3. Create GitHub release
```bash
git tag vX.Y.Z && git push origin vX.Y.Z
gh release create vX.Y.Z --title "vX.Y.Z" --generate-notes
```

### 4. Wait for CI to complete
- The `Release` workflow triggers on tag push (builds WASM, Docker image, Helm chart)
- **You MUST wait for CI to fully complete before proceeding to step 5.** Do NOT update the flux repo until CI succeeds.
- Get the run ID and watch it in the background:
```bash
gh run list --workflow=release.yml --limit 1
gh run watch <run-id>   # blocks until complete (~15 min)
```
- If CI fails: inspect logs with `gh run view <run-id> --log-failed`, fix, and **bump to a new version** (never reuse tags). Then restart from step 2.
- **Only proceed to step 5 after CI reports success.**

### 5. Update flux repo (only after CI succeeds)
Edit `../frank-personal-server/flux/apps/tafels/helmrelease.yaml`:
- Update `spec.chart.spec.version` to the new version (without `v` prefix, e.g. `0.1.23`)
- Commit and push:
```bash
cd ../frank-personal-server
git add flux/apps/tafels/helmrelease.yaml
git commit -m "chore: bump tafels to vX.Y.Z"
git push
```

## Common Issues
- **Flux won't pull new image if chart version unchanged** — always bump version, never recreate tags
- **Never update flux repo before CI completes** — the Helm chart won't exist yet, causing Flux reconciliation errors
