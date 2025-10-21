@echo off
echo Starting test: Run for 5 seconds, then resize window multiple times
timeout /t 5 /nobreak
echo.
echo Test complete! Application should still be running without crashes.
echo Please manually resize the window to verify stability.
pause
