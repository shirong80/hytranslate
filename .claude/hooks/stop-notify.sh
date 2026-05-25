#!/bin/bash
# Claude Code Stop 훅 - 작업 완료 알림 (무한루프 방지 포함)
INPUT=$(cat)

STOP_ACTIVE=$(echo "$INPUT" | jq -r '.stop_hook_active // false')

# 이미 Stop 훅에서 재진입한 경우 무한루프 방지
if [ "$STOP_ACTIVE" = "true" ]; then
  exit 0
fi

# last_assistant_message에서 요약 추출 (첫 100자)
RAW_MSG=$(echo "$INPUT" | jq -r '.last_assistant_message // "작업이 완료되었습니다"')
MESSAGE=$(echo "$RAW_MSG" | head -c 100)

# 메시지가 잘린 경우 말줄임 추가
if [ ${#RAW_MSG} -gt 100 ]; then
  MESSAGE="${MESSAGE}…"
fi

terminal-notifier \
  -title "Claude Code" \
  -subtitle "작업 완료" \
  -message "$MESSAGE" \
  -sound default \
  -group "claude-code"

exit 0
