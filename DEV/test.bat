@echo off
setlocal

if "%~1"=="" (
    echo Usage: %~nx0 ^<CPA_API_KEY^>
    exit /b 2
)

set "CPA_API_KEY=%~1"

pushd "%~dp0..\codex-rs" || exit /b 1
cargo run --bin codex -- ^
    --yolo ^
    -c "model_provider='cpa'" ^
    -c "model_providers.cpa.name='OpenAI'" ^
    -c "model_providers.cpa.base_url='https://codex.pchuan.top/v1'" ^
    -c "model_providers.cpa.env_key='CPA_API_KEY'" ^
    -c "model_providers.cpa.wire_api='responses'" ^
    -c "analytics.enabled=false" ^
    -c "feedback.enabled=false" ^
    -c "otel.metrics_exporter='none'"
set "exit_code=%ERRORLEVEL%"
popd

exit /b %exit_code%
