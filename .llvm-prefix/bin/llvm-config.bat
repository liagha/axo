@echo off
setlocal enabledelayedexpansion

if "%TARGET%"=="" (
  echo unsupported target: TARGET is not set 1>&2
  exit /b 1
)

set REPO_DIR=%~dp0\..
set REPO_DIR=%REPO_DIR:\.llvm-prefix\bin\..=%

if "%TARGET%"=="x86_64-unknown-linux-gnu" set TARGET_DIR=x86_64-linux
if "%TARGET%"=="aarch64-unknown-linux-gnu" set TARGET_DIR=aarch64-linux
if "%TARGET%"=="x86_64-apple-darwin" set TARGET_DIR=x86_64-macos
if "%TARGET%"=="aarch64-apple-darwin" set TARGET_DIR=aarch64-macos

if "%TARGET_DIR%"=="" (
  echo unsupported target: %TARGET% 1>&2
  exit /b 1
)

set AXO_LIB_DIR=%REPO_DIR%\llvm\%TARGET_DIR%\lib

if "%1"=="--libdir" (
  echo %AXO_LIB_DIR%
  exit /b 0
)

if "%REAL_LLVM_CONFIG%"=="" (
  set REAL_LLVM_CONFIG=llvm-config-19
)

%REAL_LLVM_CONFIG% %*
exit /b %ERRORLEVEL%
