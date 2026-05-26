# Translation Quality Eval Set

> PRD §14.1 — 한국어 40 / 중국어 간체 40 / 중국어 번체 20 = 총 100 샘플
> 도메인 5종: 일상회화, 비즈니스/이메일, 기술/IT, 학술/논문, 법률
> 평가 척도: 1 ~ 5 (정확성, 자연스러움, 용어 보존)

## v1 합격선 (PRD §14.2)

- 전체 평균 ≥ 4.0
- 치명적 오역률 ≤ 5%
- 법률/학술 용어 보존 실패 ≤ 10%
- 언어별 평균 ≥ 3.8

## 현재 진행 상황 (2026-05-26)

> v1.0 출시 전 100건 채점은 본 follow-up 범위 외로 별도 트래킹. 본 표는 골조 + 대표 10개
> 샘플로, `source_text` / `reference_en` 만 채워져 있고 점수는 비어 있다. 채점자 배정과
> 일정은 별도 결정.

## 샘플 표

| #   | 언어   | 도메인        | source_text                                                                    | reference_en                                                                                       | hy-mt2-7b | hy-mt2-1.8b | reviewer | note |
| --- | ------ | ------------- | ------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------- | --------- | ----------- | -------- | ---- |
| 1   | 한국어 | 일상회화      | 오늘 저녁 같이 영화 보러 갈래요?                                               | Want to go see a movie together tonight?                                                           |           |             |          |      |
| 2   | 한국어 | 비즈니스/이메일 | 회의 시간을 다음 주 화요일 오후 3시로 변경 가능하실까요?                       | Would it be possible to reschedule the meeting to 3 PM next Tuesday?                               |           |             |          |      |
| 3   | 한국어 | 기술/IT       | 이 함수는 입력값이 비어 있을 때 null 을 반환합니다.                            | This function returns null when the input is empty.                                                |           |             |          |      |
| 4   | 한국어 | 법률          | 본 계약은 양 당사자 간 합의에 따라 언제든지 해지할 수 있다.                    | This agreement may be terminated at any time by mutual consent of the parties.                     |           |             |          |      |
| 5   | 간체   | 일상회화      | 你最近怎么样? 好久没见了。                                                     | How have you been lately? Long time no see.                                                        |           |             |          |      |
| 6   | 간체   | 비즈니스/이메일 | 请尽快确认会议的具体时间, 以便我们安排后续工作。                               | Please confirm the meeting time as soon as possible so we can plan the follow-up work.             |           |             |          |      |
| 7   | 간체   | 학술/논문     | 该研究表明深度学习模型在低资源语言上的表现仍有较大改进空间。                   | This study shows that deep-learning models still have significant room for improvement in low-resource languages. |           |             |          |      |
| 8   | 간체   | 기술/IT       | 该接口在请求超时后会自动重试三次。                                             | This API automatically retries three times after a request times out.                              |           |             |          |      |
| 9   | 번체   | 일상회화      | 我等一下會去買晚餐, 你想吃什麼?                                                | I'll grab dinner in a bit — what do you want to eat?                                               |           |             |          |      |
| 10  | 번체   | 비즈니스/이메일 | 附件是本季度的銷售報告, 麻煩您過目並提供回饋。                                 | Attached is the sales report for this quarter; please review and share your feedback.              |           |             |          |      |

> 채점자가 100건을 마저 채울 때까지 PRD §19 의 "품질 평가셋 기준 만족" bullet 은 미충족.
