#!/bin/bash
# Claude Code Notification 훅 - stdin JSON에서 동적 메시지 추출 후 terminal-notifier로 알림
INPUT=$(cat)

TITLE=$(echo "$INPUT" | jq -r '.title // "Claude Code"')
MESSAGE=$(echo "$INPUT" | jq -r '.message // "알림이 도착했습니다"')
TYPE=$(echo "$INPUT" | jq -r '.notification_type // "unknown"')

# notification_type에 따라 subtitle 결정
case "$TYPE" in
  permission_prompt) SUBTITLE="권한 요청" ;;
  idle_prompt)       SUBTITLE="입력 대기중" ;;
  auth_success)      SUBTITLE="인증 완료" ;;
  elicitation_dialog) SUBTITLE="정보 입력 필요" ;;
  *)                 SUBTITLE="알림" ;;
esac

terminal-notifier \
  -title "$TITLE" \
  -subtitle "$SUBTITLE" \
  -message "$MESSAGE" \
  -sound default \
  -group "claude-code"

exit 0
