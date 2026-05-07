#!/usr/bin/env bash
# release.sh — 一鍵 commit + 自動遞增 patch tag + push
#
# 用法：
#   bash release.sh                  # commit message 自動用日期時間
#   bash release.sh "fix: 修了什麼"  # 指定 commit message
#
# 效果：
#   git add -A  →  git commit  →  git tag vX.Y.(Z+1)  →  git push origin main --tags
#
# 版本規則：
#   只遞增 patch（0.1.1 → 0.1.2 → 0.1.3 …）
#   若需升 minor/major，手動 git tag v1.0.0 一次即可，之後 patch 繼續從 0 累加

set -euo pipefail

# ── 1. Commit message ─────────────────────────────────────────────────────────
MSG="${1:-chore: release $(date '+%Y-%m-%d %H:%M')}"

# ── 2. 計算下一個 tag ─────────────────────────────────────────────────────────
LATEST=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")

# 去掉前綴 v，分解三段
VERSION="${LATEST#v}"
IFS='.' read -r MAJOR MINOR PATCH <<< "$VERSION"
PATCH=$(( PATCH + 1 ))
NEW_TAG="v${MAJOR}.${MINOR}.${PATCH}"

echo "────────────────────────────────────"
echo "  Last tag  : $LATEST"
echo "  New tag   : $NEW_TAG"
echo "  Message   : $MSG"
echo "────────────────────────────────────"

# ── 3. git add ────────────────────────────────────────────────────────────────
git add -A

# ── 4. commit（若有變更才 commit，只 tag 也 OK）────────────────────────────────
if git diff --cached --quiet; then
  echo "⚠️  No staged changes — skipping commit, still tagging & pushing"
else
  git commit -m "$MSG"
fi

# ── 5. tag + push ─────────────────────────────────────────────────────────────
git tag "$NEW_TAG"
git push origin main --tags

echo ""
echo "✅  $NEW_TAG pushed → GitHub Actions building Windows installer..."
echo "    https://github.com/rex4ssd/richness_win11/actions"
