#!/usr/bin/env bash

# Usage: ./annotate-pr-benchmark-link.sh <branch-name>

BRANCH=$1

# Get latest, successful runs for master and $BRANCH
MASTER_RUN=$(gh api "/repos/maximumstock/dns-thingy/actions/runs?branch=master&status=completed" | jq '[.workflow_runs[]|select(.conclusion=="success")][0]')
BRANCH_RUN=$(gh api "/repos/maximumstock/dns-thingy/actions/runs?branch=$BRANCH&status=completed" | jq '[.workflow_runs[]|select(.conclusion=="success")][0]')

MASTER_SUITE_ID=$(echo "$MASTER_RUN" | jq ".check_suite_id")
BRANCH_SUITE_ID=$(echo "$BRANCH_RUN" | jq ".check_suite_id")

MASTER_RUN_ID=$(echo "$MASTER_RUN" | jq ".id")
BRANCH_RUN_ID=$(echo "$BRANCH_RUN" | jq ".id")

MASTER_ARTIFACT_ID=$(gh api "/repos/maximumstock/dns-thingy/actions/runs/$MASTER_RUN_ID/artifacts" | jq ".artifacts[0].id")
BRANCH_ARTIFACT_ID=$(gh api "/repos/maximumstock/dns-thingy/actions/runs/$BRANCH_RUN_ID/artifacts" | jq ".artifacts[0].id")

MASTER_BENCHMARKS_URL="https://github.com/maximumstock/dns-thingy/suites/$MASTER_SUITE_ID/artifacts/$MASTER_ARTIFACT_ID"
BRANCH_BENCHMARKS_URL="https://github.com/maximumstock/dns-thingy/suites/$BRANCH_SUITE_ID/artifacts/$BRANCH_ARTIFACT_ID"

echo "Master Benchmark: $MASTER_BENCHMARKS_URL"
echo "Branch Benchmark: $BRANCH_BENCHMARKS_URL"

PR_NUMBER=$(gh api "/repos/maximumstock/dns-thingy/pulls?head=$BRANCH&per_page=1" | jq ".[0].number")

gh pr comment $PR_NUMBER --body "
  - Master Benchmark: $MASTER_BENCHMARKS_URL
  - Branch Benchmark: $BRANCH_BENCHMARKS_URL
  "
