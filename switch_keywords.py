#!/usr/bin/env python3
import sys
import time
import shutil
import subprocess
from datetime import datetime


def git_commit_push(message):
    subprocess.run(["git", "add", "keyword.toml"], check=True)
    subprocess.run(["git", "commit", "-m", message], check=True)
    subprocess.run(["gh", "repo", "sync", "--source", "origin", "--force"], capture_output=True)
    subprocess.run(["git", "push"], check=True)


def apply_strict():
    shutil.copy("keyword_strict.toml", "keyword.toml")
    print(f"[{datetime.now().strftime('%H:%M:%S')}] keyword.toml <- keyword_strict.toml")
    git_commit_push("chore: apply strict keyword filter")


def apply_rough():
    shutil.copy("keyword_rough.toml", "keyword.toml")
    print(f"[{datetime.now().strftime('%H:%M:%S')}] keyword.toml <- keyword_rough.toml")
    git_commit_push("chore: revert to rough keyword filter")


def parse_time(s):
    try:
        return datetime.strptime(s.strip(), "%H:%M").replace(
            year=datetime.now().year,
            month=datetime.now().month,
            day=datetime.now().day,
        )
    except ValueError:
        print("시간 형식이 잘못됐습니다. 예: 14:00")
        sys.exit(1)


def main():
    if len(sys.argv) > 1:
        time_str = sys.argv[1]
    else:
        time_str = input("복원 시간을 입력하세요 (예: 14:00): ")

    revert_at = parse_time(time_str)
    now = datetime.now()

    if revert_at <= now:
        print("입력한 시간이 이미 지났습니다. 오늘 이후로 다시 실행해 주세요.")
        sys.exit(1)

    print(f"strict 적용 후 {revert_at.strftime('%H:%M')}에 rough로 복원합니다.")

    # Step 1: strict 적용
    apply_strict()

    # Step 2: 지정 시간까지 대기
    wait_sec = (revert_at - datetime.now()).total_seconds()
    print(f"{wait_sec:.0f}초 후에 rough로 복원됩니다...")
    time.sleep(max(0, wait_sec))

    # Step 3: rough 복원
    apply_rough()
    print("완료.")


if __name__ == "__main__":
    main()
