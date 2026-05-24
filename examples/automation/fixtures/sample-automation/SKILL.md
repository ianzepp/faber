+++
version = 1
id = "sample-automation"
kind = "cron"
name = "Sample Automation"
provider = "openai"
execute = "codex"
status = "PAUSED"
rrule = "RRULE:FREQ=HOURLY;INTERVAL=1;BYMINUTE=0"
model = "gpt-5.4-mini"
reasoning_effort = "high"
cwds = ["/tmp/faber-automation-fixture"]
created_at = 1778148984162
updated_at = 1778210342668
+++

Use this fixture to validate Faber automation metadata parsing and dry-run prompt assembly.

Do not execute external commands from this fixture.
