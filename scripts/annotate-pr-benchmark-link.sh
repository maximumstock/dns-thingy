#!/usr/bin/env bash

# Usage: ./annotate-pr-benchmark-link.sh <branch-name>

BRANCH=$1

# Get latest, successful run for master
MASTER_RUN=$(gh api "/repos/maximumstock/dns-thingy/actions/runs?branch=master&status=completed" | jq '[.workflow_runs[]|select(.conclusion=="success")][0]')
# Get latest run for $BRANCH as it most likely contains the latest, succesful benchmark job
echo "BRANCH $BRANCH"
echo "BRANCH RUN URI /repos/maximumstock/dns-thingy/actions/runs?branch=$BRANCH&status=completed"
BRANCH_RUN=$(gh api "/repos/maximumstock/dns-thingy/actions/runs?branch=$BRANCH&status=completed" | jq '.workflow_runs[0]')

MASTER_SUITE_ID=$(echo "$MASTER_RUN" | jq ".check_suite_id")
BRANCH_SUITE_ID=$(echo "$BRANCH_RUN" | jq ".check_suite_id")

echo "BRANCH SUITE ID $BRANCH_SUITE_ID"

MASTER_RUN_ID=$(echo "$MASTER_RUN" | jq ".id")
BRANCH_RUN_ID=$(echo "$BRANCH_RUN" | jq ".id")

echo "BRANCH RUN ID $BRANCH_RUN_ID"

MASTER_ARTIFACT_ID=$(gh api "/repos/maximumstock/dns-thingy/actions/runs/$MASTER_RUN_ID/artifacts" | jq ".artifacts[0].id")
BRANCH_ARTIFACT_ID=$(gh api "/repos/maximumstock/dns-thingy/actions/runs/$BRANCH_RUN_ID/artifacts" | jq ".artifacts[0].id")

echo "BRANCH ARTIFACT ID $BRANCH_ARTIFACT_ID"

MASTER_BENCHMARKS_URL="https://github.com/maximumstock/dns-thingy/suites/$MASTER_SUITE_ID/artifacts/$MASTER_ARTIFACT_ID"
BRANCH_BENCHMARKS_URL="https://github.com/maximumstock/dns-thingy/suites/$BRANCH_SUITE_ID/artifacts/$BRANCH_ARTIFACT_ID"

echo "Master Benchmark: $MASTER_BENCHMARKS_URL"
echo "Branch Benchmark: $BRANCH_BENCHMARKS_URL"

PR_NUMBER=$(gh api "/repos/maximumstock/dns-thingy/pulls?head=$BRANCH&per_page=1" | jq ".[0].number")

gh run download $MASTER_RUN_ID -n benchmark-results --dir master-benchmark-results
gh run download $BRANCH_RUN_ID -n benchmark-results --dir branch-benchmark-results

MASTER_BASIC_STDOUT=$(cat master-benchmark-results/basic/stdout)
MASTER_THREADED_STDOUT=$(cat master-benchmark-results/threaded-4/stdout)
MASTER_TOKIO_STDOUT=$(cat master-benchmark-results/tokio/stdout)
BRANCH_BASIC_STDOUT=$(cat branch-benchmark-results/basic/stdout)
BRANCH_THREADED_STDOUT=$(cat branch-benchmark-results/threaded-4/stdout)
BRANCH_TOKIO_STDOUT=$(cat branch-benchmark-results/tokio/stdout)

gh pr comment $PR_NUMBER --body "
  - Master Benchmark: $MASTER_BENCHMARKS_URL
  - Branch Benchmark: $BRANCH_BENCHMARKS_URL

  # Basic Old

  \`\`\`
  $MASTER_BASIC_STDOUT
  \`\`\`

  # Basic New

  \`\`\`
  $BRANCH_BASIC_STDOUT
  \`\`\`

  ## Threaded Old

  \`\`\`
  $MASTER_THREADED_STDOUT
  \`\`\`

  ## Threaded New

  \`\`\`
  $BRANCH_THREADED_STDOUT
  \`\`\`

  ## Tokio Old

  \`\`\`
  $MASTER_TOKIO_STDOUT
  \`\`\`

  ## Tokio New

  \`\`\`
  $BRANCH_TOKIO_STDOUT
  \`\`\`
  "
